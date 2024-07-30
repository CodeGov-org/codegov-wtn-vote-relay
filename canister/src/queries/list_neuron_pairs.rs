use crate::{state, NeuronPairPublic};
use ic_cdk::query;

#[query]
fn list_neuron_pairs() -> Vec<NeuronPairPublic> {
    state::read(|s| s.list_neuron_pairs())
}
