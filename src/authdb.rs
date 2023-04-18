use super::Db;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::{executor, future};
use serde::{Deserialize, Serialize};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_sqldb::{minicbor, QueryResult, SqlDb, SqlDbError, Statement};

// begin by DBA
// maintenance role: dbadmin@enterprise.com

const POS_DB: &str = "pos";
const AUTH_SESSIONS_TABLE: &str = "auths";
const STAFFS_TABLE: &str = "staffs";
const NODES_TABLE: &str = "nodes";

const DBD_STAFFS: &str = "create table if not exists staffs (
    id BIGSERIAL PRIMARY KEY,
    username varchar(50) not null,
    passhash varchar(200) not null,
    email varchar(200) not null
);";
const DBD_NODES: &str = "create table if not exists nodes (
    id BIGSERIAL PRIMARY KEY,
    uuid varchar(200) not null,
    label varchar(200) not null
);";
const DBD_AUTH_SESSIONS: &str = "create table if not exists auths (
    id BIGSERIAL PRIMARY KEY,
    token varchar(200) not null,
    staff_id bigint not null,
    node_id bigint not null,
    expires TIMESTAMP DEFAULT now()
);";

/*
 * format argument must be a string literal
const INIT_STAFFS: &str = "INSERT INTO {} (id, username, passhash, email) VALUES ('admin', 'captaincosmo', 'captain@creator-ms.io') ON CONFLICT DO NOTHING;
INSERT INTO staffs (id, username, passhash) VALUES ('user', 'hellocosmo', 'user@creator-ms.io') ON CONFLICT DO NOTHING; ";
const INIT_NODES: &str = "INSERT INTO {} (id, uuid, label) VALUES ('com.posnode.ocean.fs01', 'ocean city flag ship store') ON CONFLICT DO NOTHING;
INSERT INTO nodes (id, uuid, label) VALUES ('com.posnode.stargate.ap02', 'stargate airport gift shop') ON CONFLICT DO NOTHING;";

const SELECT_STAFF: &str = "SELECT id, passhash FROM {} WHERE username = {}";

const SELECT_NODE: &str = "SELECT id FROM {} WHERE uuid = {}";
*/
// end by DBA

#[derive(Default, Serialize, Deserialize, minicbor::Decode, Clone)]
pub(crate) struct DbMinStaff {
    #[n(0)]
    pub id: u64,
    #[n(1)]
    pub passhash: String,
}

#[derive(Default, Serialize, Deserialize, minicbor::Decode, Clone)]
pub(crate) struct DbMinNode {
    #[n(0)]
    pub id: u64,
}

#[derive(Default, Serialize, Deserialize, minicbor::Decode, Clone)]
pub(crate) struct AuthSession {
    #[n(0)]
    pub id: u64,
    #[n(1)]
    pub token: String,
    #[n(2)]
    pub staffid: u64,
    #[n(3)]
    pub nodeid: u64,
    // #[n(4)]
    // pub expires: u64,
}

async fn fetch_node_id(
    ctx: &Context,
    client: &Db,
    nodeuuid: String
) -> Result<u64, SqlDbError> {

    let resp = client.query(
        ctx, 
        &Statement{
            sql: format!("SELECT id FROM {} WHERE uuid = {}", NODES_TABLE, nodeuuid),
            database: Some(POS_DB.to_string()),
            ..Default::default()
        },
    ).await?;

    let rows: Vec<DbMinNode> = safe_decode(&resp)?;
    if rows.len() < 1 {
        return Ok(0);
    }
    return Ok(rows.first().unwrap().id);
}

async fn fetch_staff_id(
    ctx: &Context,
    client: &Db,
    username: String,
    untrust_hash: String
) -> Result<u64, SqlDbError> {

    let resp = client.query(
        ctx, 
        &Statement{
            // SQL injection risk on username, still need a valid password
            sql: format!("SELECT id, passhash FROM {} WHERE username = '{}'", STAFFS_TABLE, username),
            database: Some(POS_DB.to_string()),
            ..Default::default()
        },
    )
    .await?;

    let rows: Vec<DbMinStaff> = safe_decode(&resp)?;
    if rows.len() < 1 {
        return Err(SqlDbError::new(
            "notfound",
            "no such user".to_string(),
        ));
    }

    let min_staff = rows.first().unwrap();
    // TODO: do a hash algo
    if ! min_staff.passhash.eq(&untrust_hash) {
        return Err(SqlDbError::new(
            "invalid",
            "invalid password".to_string(),
        ));
    }
    Ok(min_staff.id)
}

async fn insert_token(
    ctx: &Context,
    client: &Db,
    node_id: u64,
    staff_id: u64
)-> Result<String, SqlDbError> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_the_epoch.as_secs() * 1000 +
        since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    let token = (format!("{}-{}", node_id, in_ms)).to_string();
    // can test if the token exists, any possible duplicate old token is the same POS node and rarely duplicate
    match client.execute(
        ctx, 
        &Statement{
            // all data is from code, no injection risk
            sql: format!("REPLACE INTO {} (token, node_id, staff_id, expires) VALUES ('{}',{},{})", AUTH_SESSIONS_TABLE, token, node_id, staff_id),
            database: Some(POS_DB.to_string()),
            ..Default::default()
        },
    ).await {
        Ok(_) => Ok(token),
        Err(e) => Err(e.into())
    }
}


pub(crate) async fn staff_login (
    ctx: &Context,
    client: &Db,
    username: String,
    untrust_hash: String, // just remind front end developer not to 
    nodeuuid: String
) -> Result<String,SqlDbError>
{
    let (r1, r2) = future::join(
        fetch_node_id(ctx, client, nodeuuid), 
        fetch_staff_id(ctx, client, username, untrust_hash),
    ).await;

    match (r1, r2) {
        (Ok(node_id), Ok(staff_id)) => {
            match insert_token(ctx, client, node_id, staff_id).await {
                Ok(t) => Ok(t),
                Err(e) => Err(e.into())
            }
        },
        (Err(e), _) | (_, Err(e)) => {
            Err(e)
        }
    }
}


// impl From<DbMinStaff> for petclinic_interface::Visit {
//     fn from(source: DbVisit) -> Self {
//         petclinic_interface::Visit {
//             date: petclinic_interface::Date {
//                 day: source.day as _,
//                 month: source.month as _,
//                 year: source.year as _,
//             },
//             description: source.description,
//             pet_id: source.petid,
//             time: petclinic_interface::Time {
//                 hour: source.hour as _,
//                 minute: source.minute as _,
//             },
//             vet_id: source.vetid,
//         }
//     }
// }

/// When using this to decode Vecs, will get an empty vec
/// as a response when no rows are returned
fn safe_decode<'b, T>(resp: &'b QueryResult) -> Result<Vec<T>, minicbor::decode::Error>
where
    T: Default + minicbor::Decode<'b, ()>,
{
    if resp.num_rows == 0 {
        Ok(Vec::new())
    } else {
        wasmbus_rpc::minicbor::decode(&resp.rows)
    }
}

pub(crate) async fn ensure_db(
    ctx: &Context,
    client: &Db,
) -> Result<(), SqlDbError>{
    let (r1, r2, r3) = future::join3(
        ensure_table_staffs(ctx, client), 
        ensure_table_nodes(ctx, client),
        ensure_table_auth_sessions(ctx, client),
    ).await;

    match (r1, r2, r3) {
        (Ok(_), Ok(_), Ok(_)) => {
            Ok(())
        }
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            Err(e)
        }
    }
}

async fn ensure_table_staffs(
    ctx: &Context,
    client: &Db,
) -> Result<(), SqlDbError>{
    let resp = client.execute(
        ctx,
        &Statement{
            sql: format!( "create table if not exists {} (
                id BIGSERIAL PRIMARY KEY,
                username varchar(50) not null,
                passhash varchar(200) not null,
                email varchar(200) not null
            );", STAFFS_TABLE),
            database: Some(POS_DB.to_string()),
        ..Default::default()
        }
    ).await?;
    match resp.error {
        None => {
            let resp2 = client.execute(
                ctx,
                &Statement{
                    sql: format!("INSERT INTO {} (id, username, passhash, email) VALUES ('admin', 'captaincosmo', 'captain@creator-ms.io'), ('user','nicecosmo', 'user@creator-ms.io') ON CONFLICT DO NOTHING", STAFFS_TABLE),
                    database: Some(POS_DB.to_string()),
                ..Default::default()
                }
            ).await?;
            match resp.error{
                None => Ok(()),
                Some(e) => Err(e),
            }
        },
        Some(e) => Err(e),
    }
}

async fn ensure_table_nodes(
    ctx: &Context,
    client: &Db,
) -> Result<(), SqlDbError>{
    let resp = client.execute(
        ctx,
        &Statement{
            sql: DBD_NODES.to_string(),
            database: Some(POS_DB.to_string()),
        ..Default::default()
        }
    ).await?;
    match resp.error {
        None => {
            let resp2 = client.execute(
                ctx,
                &Statement{
                    sql: format!("INSERT INTO {} (id, uuid, label) VALUES (1, 'com.posnode.ocean.fs01', 'ocean city flag ship store'),(2, 'com.posnode.stargate.ap02', 'stargate airport gift shop') ON CONFLICT DO NOTHING", NODES_TABLE),
                    database: Some(POS_DB.to_string()),
                ..Default::default()
                }
            ).await?;
            match resp.error{
                None => Ok(()),
                Some(e) => Err(e),
            }
        },
        Some(e) => Err(e),
    }
}

async fn ensure_table_auth_sessions(
    ctx: &Context,
    client: &Db,
) -> Result<(), SqlDbError>{
    let resp = client.execute(
        ctx,
        &Statement{
            sql: DBD_AUTH_SESSIONS.to_string(),
            database: Some(POS_DB.to_string()),
        ..Default::default()
        }
    ).await?;
    match resp.error{
        None => Ok(()),
        Some(e) => Err(e),
    }

}

// fn random_string_generate<S: AsRef<str>>(length: usize, charset: S) -> String {
//     let charset_str = charset.as_ref();

//     if charset_str.is_empty() {
//         panic!("Provided charset is empty! It should contain at least one character");
//     }

//     let chars: Vec<char> = charset_str.chars().collect();
//     let mut result = String::with_capacity(length);

//     unsafe {
//         for _ in 0..length {
//             result.push(
//                 *chars.get_unchecked(fastrand::usize(0..chars.len()))
//             );
//         }
//     }

//     result
// }