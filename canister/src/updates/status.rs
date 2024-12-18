use crate::{CanisterIdRecord, CanisterStatusResponse};
use ic_cdk::update;

#[update]
async fn status() -> CanisterStatusResponse {
    let canister_id = ic_cdk::id();
    ic_cdk::api::management_canister::main::canister_status(CanisterIdRecord { canister_id })
        .await
        .unwrap()
        .0
}
