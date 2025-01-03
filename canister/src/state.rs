use crate::logs::log;
use crate::neuron_pair::NeuronPair;
use crate::{InitArgs, NnsVote, VoteToProcess, WtnVote};
use ic_principal::Principal;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::btree_map::Entry::{Occupied, Vacant};
use std::collections::{BTreeMap, VecDeque};

const DEFAULT_NNS_GOVERNANCE_CANISTER_ID: Principal =
    Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);
const DEFAULT_WTN_GOVERNANCE_CANISTER_ID: Principal =
    Principal::from_slice(&[0, 0, 0, 0, 2, 0, 0, 214, 1, 1]);
const DEFAULT_WTN_PROTOCOL_CANISTER_ID: Principal =
    Principal::from_slice(&[0, 0, 0, 0, 2, 48, 1, 106, 1, 1]);

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    nns_governance_canister_id: Principal,
    wtn_governance_canister_id: Principal,
    wtn_protocol_canister_id: Principal,
    neuron_pairs: BTreeMap<u64, NeuronPair>,
    votes_to_process: VecDeque<VoteToProcess>,
    #[serde(default)]
    cached_wtn_proposals_per_nns_proposal: BTreeMap<u64, Option<u64>>,
}

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

pub fn init(state: State) {
    STATE.with_borrow_mut(|s| {
        if s.is_some() {
            panic!("{}", STATE_ALREADY_INITIALIZED);
        } else {
            *s = Some(state);
        }
    });
}

pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect(STATE_NOT_INITIALIZED)))
}

pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with_borrow_mut(|s| f(s.as_mut().expect(STATE_NOT_INITIALIZED)))
}

pub fn take() -> State {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl State {
    pub fn new(args: InitArgs) -> State {
        State {
            nns_governance_canister_id: args
                .nns_governance_canister_id
                .unwrap_or(DEFAULT_NNS_GOVERNANCE_CANISTER_ID),
            wtn_governance_canister_id: args
                .wtn_governance_canister_id
                .unwrap_or(DEFAULT_WTN_GOVERNANCE_CANISTER_ID),
            wtn_protocol_canister_id: args
                .wtn_protocol_canister_id
                .unwrap_or(DEFAULT_WTN_PROTOCOL_CANISTER_ID),
            neuron_pairs: BTreeMap::new(),
            votes_to_process: VecDeque::new(),
            cached_wtn_proposals_per_nns_proposal: BTreeMap::new(),
        }
    }

    pub fn nns_governance_canister_id(&self) -> Principal {
        self.nns_governance_canister_id
    }

    pub fn wtn_governance_canister_id(&self) -> Principal {
        self.wtn_governance_canister_id
    }

    pub fn wtn_protocol_canister_id(&self) -> Principal {
        self.wtn_protocol_canister_id
    }

    pub fn register_neuron_pair(
        &mut self,
        caller: Principal,
        name: String,
        nns_neuron_id: u64,
        wtn_neuron_id: [u8; 32],
    ) -> Option<u64> {
        let pair = NeuronPair::new(name, caller, nns_neuron_id, wtn_neuron_id);
        let id = pair.id();
        match self.neuron_pairs.entry(id) {
            Vacant(e) => {
                e.insert(pair);
                Some(id)
            }
            _ => None,
        }
    }

    pub fn deregister_neuron_pair(&mut self, caller: Principal, pair_id: u64) -> bool {
        match self.neuron_pairs.entry(pair_id) {
            Occupied(e) => {
                if e.get().admin() == caller {
                    e.remove();
                    self.votes_to_process.retain(|v| v.pair_id() != pair_id);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn neuron_pairs(&self) -> &BTreeMap<u64, NeuronPair> {
        &self.neuron_pairs
    }

    pub fn record_nns_vote(&mut self, pair_id: u64, vote: NnsVote) {
        if let Some(pair) = self.neuron_pairs.get_mut(&pair_id) {
            if pair.is_newly_seen_nns_vote(vote.proposal_id) {
                self.push_vote_to_process(VoteToProcess::NnsVote(pair_id, vote));
            }
        }
    }

    pub fn record_wtn_vote_registered(&mut self, pair_id: u64, vote: WtnVote) {
        if let Some(pair) = self.neuron_pairs.get_mut(&pair_id) {
            log(format!("WTN vote registered: {vote:?}. PairId: {pair_id}"));
            pair.record_wtn_vote_registered(vote);
        }
    }

    pub fn push_vote_to_process(&mut self, vote: VoteToProcess) {
        log(format!("Vote queued for processing: {vote:?}"));
        self.votes_to_process.push_back(vote);
        crate::jobs::process_votes::start_job_if_required(self);
    }

    pub fn pop_next_vote_to_process(&mut self) -> Option<VoteToProcess> {
        self.votes_to_process.pop_front()
    }

    pub fn votes_to_process(&self) -> Vec<VoteToProcess> {
        self.votes_to_process.iter().cloned().collect()
    }

    pub fn votes_to_process_count(&self) -> usize {
        self.votes_to_process.len()
    }

    pub fn record_wtn_proposal_for_nns_proposal(
        &mut self,
        nns_proposal_id: u64,
        wtn_proposal_id: Option<u64>,
    ) {
        self.cached_wtn_proposals_per_nns_proposal
            .insert(nns_proposal_id, wtn_proposal_id);

        while self.cached_wtn_proposals_per_nns_proposal.len() > 500 {
            self.cached_wtn_proposals_per_nns_proposal.pop_first();
        }
    }

    pub fn get_cached_wtn_proposal_for_nns_proposal(
        &self,
        nns_proposal_id: u64,
    ) -> Option<Option<u64>> {
        self.cached_wtn_proposals_per_nns_proposal
            .get(&nns_proposal_id)
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canister_ids() {
        assert_eq!(
            DEFAULT_NNS_GOVERNANCE_CANISTER_ID,
            Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
        );
        assert_eq!(
            DEFAULT_WTN_GOVERNANCE_CANISTER_ID,
            Principal::from_text("jfnic-kaaaa-aaaaq-aadla-cai").unwrap()
        );
        assert_eq!(
            DEFAULT_WTN_PROTOCOL_CANISTER_ID,
            Principal::from_text("tsbvt-pyaaa-aaaar-qafva-cai").unwrap()
        );
    }
}
