use pos_interface::{
    PosAuthReceiver, PosAuth, TokenAuthRequest, AuthResult, LoginRequest, TokenResult, EmptyObj
};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_sqldb::SqlDbSender;

pub(crate) type Db = SqlDbSender<WasmHost>;

mod authdb;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, PosAuth)]
struct PosAuthActor {}

#[async_trait]
impl PosAuth for PosAuthActor {

    async fn init_db(
        &self, 
        ctx: &Context, 
        req: &EmptyObj
    ) -> RpcResult<EmptyObj> {
        let sdb = SqlDbSender::new();
        authdb::ensure_db(ctx, &sdb);
        // Ok(match sdb::ensure_db().await {
        //     Ok(_) => "done".to_string(),
        //     Err(e) => {
        //         format!("Failed to record visit: {}", e).to_string();
        //     }
        // })
        Ok(req.clone())
    }

    async fn auth_by_token(
        &self, 
        ctx: &Context, 
        req: &TokenAuthRequest
    ) -> RpcResult<AuthResult> {
        let sdb = SqlDbSender::new();
        let req = req.clone();
        // let orderId,e = authdb::store_order(req)
        let value: AuthResult = AuthResult{
            staff_id: 1,
            pos_id: 2,
        };
        Ok(value)
    }

    async fn staff_login(
        &self,
        ctx: &Context,
        req: &LoginRequest
    ) -> RpcResult<TokenResult> {
        let sdb = SqlDbSender::new();
        let req = req.clone();
        // let orderId,e = authdb::store_order(req)
        let value: TokenResult = TokenResult{
            token: "t1".to_string(),
        };
        Ok(value)
    }
}

// use petclinic_interface::{
//     ListVisitsRequest, RecordVisitRequest, VisitList, Visits, VisitsReceiver,
// };
// use wasmbus_rpc::actor::prelude::*;
// use wasmcloud_interface_logging::error;
// use wasmcloud_interface_sqldb::SqlDbSender;

// pub(crate) type Db = SqlDbSender<WasmHost>;

// mod db;

// #[derive(Debug, Default, Actor, HealthResponder)]
// #[services(Actor, Visits)]
// struct VisitsActor {}

// #[async_trait]
// impl Visits for VisitsActor {
//     async fn list_visits(&self, ctx: &Context, arg: &ListVisitsRequest) -> RpcResult<VisitList> {
//         let db = SqlDbSender::new();
//         let arg = arg.clone();
//         // let visits = db::list_visits_by_owner_and_pet(ctx, &db, arg.owner_id, arg.pet_ids).await?;
//         // Ok(visits.iter().cloned().map(|v| v.into()).collect())
//         Ok(VisitList::new())
//     }

//     async fn record_visit(&self, ctx: &Context, arg: &RecordVisitRequest) -> RpcResult<bool> {
//         let db = SqlDbSender::new();

//         // Ok(
//         //     match db::record_visit(ctx, &db, arg.owner_id, arg.visit.clone()).await {
//         //         Ok(_) => true,
//         //         Err(e) => {
//         //             error!("Failed to record visit: {}", e);
//         //             false
//         //         }
//         //     },
//         // )
//         Ok(true)
//     }
// }
