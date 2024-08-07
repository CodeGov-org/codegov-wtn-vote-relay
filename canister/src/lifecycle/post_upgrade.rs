use crate::lifecycle::READER_WRITER_BUFFER_SIZE;
use crate::memory::get_upgrades_memory;
use crate::state::State;
use crate::InitOrUpgradeArgs;
use ic_cdk::post_upgrade;
use ic_stable_structures::reader::{BufferedReader, Reader};
use serde::Deserialize;

#[post_upgrade]
fn post_upgrade(args: InitOrUpgradeArgs) {
    let _args = args.into_upgrade_args();
    let memory = get_upgrades_memory();
    let reader = BufferedReader::new(READER_WRITER_BUFFER_SIZE, Reader::new(&memory, 0));
    let mut deserializer = rmp_serde::Deserializer::new(reader);

    let state = State::deserialize(&mut deserializer).unwrap();

    crate::jobs::start_jobs(&state);
    crate::state::init(state);

    ic_cdk::println!("Canister upgrade complete");
}
