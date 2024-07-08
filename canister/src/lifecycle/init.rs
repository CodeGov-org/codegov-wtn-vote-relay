use crate::state::State;
use crate::{InitOrUpgradeArgs, state};
use ic_cdk::init;

#[init]
fn init(args: InitOrUpgradeArgs) {
    let init_args = args.to_init_args();

    state::init(State::new(init_args));
}
