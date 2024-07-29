use crate::state::State;
use crate::InitOrUpgradeArgs;
use ic_cdk::init;

#[init]
fn init(args: InitOrUpgradeArgs) {
    let init_args = args.into_init_args();

    let state = State::new(init_args);
    crate::jobs::start_jobs(&state);
    crate::state::init(state);

    ic_cdk::println!("Canister initialized");
}
