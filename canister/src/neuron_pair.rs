use crate::{NeuronPairPublic, WtnVote};
use candid::Deserialize;
use ic_principal::Principal;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct NeuronPair {
    id: u64,
    name: String,
    admin: Principal,
    nns_neuron_id: u64,
    wtn_neuron_id: [u8; 32],
    already_seen_nns_votes: BTreeSet<u64>,
    wtn_votes: Vec<WtnVote>,
}

impl NeuronPair {
    pub fn new(
        name: String,
        admin: Principal,
        nns_neuron_id: u64,
        wtn_neuron_id: [u8; 32],
    ) -> NeuronPair {
        let mut bytes = Vec::new();
        bytes.extend(admin.as_slice());
        bytes.extend(nns_neuron_id.to_be_bytes());
        bytes.extend(wtn_neuron_id);
        let id = bytes.chunks(8).fold(0, |c, n| c + Self::u64_from_bytes(n));

        NeuronPair {
            id,
            name,
            admin,
            nns_neuron_id,
            wtn_neuron_id,
            already_seen_nns_votes: BTreeSet::new(),
            wtn_votes: Vec::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn admin(&self) -> Principal {
        self.admin
    }

    pub fn nns_neuron_id(&self) -> u64 {
        self.nns_neuron_id
    }

    pub fn wtn_neuron_id(&self) -> [u8; 32] {
        self.wtn_neuron_id
    }

    pub fn is_newly_seen_nns_vote(&mut self, proposal_id: u64) -> bool {
        if self.already_seen_nns_votes.insert(proposal_id) {
            self.prune_old_nns_votes();
            true
        } else {
            false
        }
    }

    pub fn record_wtn_vote_registered(&mut self, vote: WtnVote) {
        self.wtn_votes.push(vote);
    }

    fn prune_old_nns_votes(&mut self) {
        while self.already_seen_nns_votes.len() > 1000 {
            self.already_seen_nns_votes.pop_first();
        }
    }

    fn u64_from_bytes(bytes: &[u8]) -> u64 {
        let mut u64_bytes = [0u8; 8];
        u64_bytes[..bytes.len()].copy_from_slice(bytes);
        u64::from_be_bytes(u64_bytes)
    }
}

impl From<&NeuronPair> for NeuronPairPublic {
    fn from(value: &NeuronPair) -> Self {
        NeuronPairPublic {
            id: value.id,
            name: value.name.clone(),
            admin: value.admin,
            nns_neuron_id: value.nns_neuron_id,
            wtn_neuron_id: value.wtn_neuron_id,
        }
    }
}
