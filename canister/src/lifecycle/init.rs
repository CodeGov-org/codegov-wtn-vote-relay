use crate::state::State;
use crate::InitOrUpgradeArgs;
use ic_cdk::init;

#[init]
fn init(args: InitOrUpgradeArgs) {
    let init_args = args.to_init_args();

    crate::state::init(State::new(init_args));
    crate::jobs::start_jobs();
}
