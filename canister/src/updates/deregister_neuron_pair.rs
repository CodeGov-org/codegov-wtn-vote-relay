use crate::{state, DeregisterNeuronPairArgs};
use ic_cdk::update;

#[update]
fn deregister_neuron_pair(args: DeregisterNeuronPairArgs) -> bool {
    let caller = ic_cdk::caller();
    state::mutate(|s| s.deregister_neuron_pair(caller, args.pair_id))
}
