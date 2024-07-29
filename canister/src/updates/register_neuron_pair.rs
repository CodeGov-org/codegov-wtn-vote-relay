use crate::{state, RegisterNeuronPairArgs};
use ic_cdk::update;

#[update]
fn register_neuron_pair(args: RegisterNeuronPairArgs) -> bool {
    let caller = ic_cdk::caller();
    state::mutate(|s| s.register_neuron_pair(caller, args.nns_neuron_id, args.wtn_neuron_id))
}