use crate::{state, NeuronPairPublic};
use ic_cdk::update;

#[update]
fn list_neuron_pairs() -> Vec<NeuronPairPublic> {
    state::read(|s| s.list_neuron_pairs())
}
