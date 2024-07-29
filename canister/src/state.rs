use crate::neuron_pair::NeuronPair;
use crate::{InitArgs, NnsVote, VoteToProcess, WtnVote};
use ic_principal::Principal;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::btree_map::Entry::Vacant;
use std::collections::{BTreeMap, VecDeque};

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

#[allow(dead_code)]
pub fn read<F: FnOnce(&State) -> R, R>(f: F) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect(STATE_NOT_INITIALIZED)))
}

#[allow(dead_code)]
pub fn mutate<F: FnOnce(&mut State) -> R, R>(f: F) -> R {
    STATE.with_borrow_mut(|s| f(s.as_mut().expect(STATE_NOT_INITIALIZED)))
}

pub fn take() -> State {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl State {
    pub fn new(args: InitArgs) -> State {
        State {
            nns_governance_canister_id: args.nns_governance_canister_id,
            wtn_governance_canister_id: args.wtn_governance_canister_id,
            wtn_protocol_canister_id: args.wtn_protocol_canister_id,
            neuron_pairs: BTreeMap::new(),
            votes_to_process: VecDeque::new(),
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
        nns_neuron_id: u64,
        wtn_neuron_id: [u8; 32],
    ) -> bool {
        let pair = NeuronPair::new(caller, nns_neuron_id, wtn_neuron_id);
        match self.neuron_pairs.entry(pair.id()) {
            Vacant(e) => {
                e.insert(pair);
                true
            }
            _ => false,
        }
    }

    pub fn neuron_pair(&self, pair_id: u64) -> Option<&NeuronPair> {
        self.neuron_pairs.get(&pair_id)
    }

    pub fn iter_neuron_pairs(&self) -> impl Iterator<Item = &NeuronPair> {
        self.neuron_pairs.values()
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
            ic_cdk::println!("WTN vote registered: {vote:?}");
            pair.record_wtn_vote_registered(vote);
        }
    }

    pub fn push_vote_to_process(&mut self, vote: VoteToProcess) {
        ic_cdk::println!("Vote queued for processing: {vote:?}");
        self.votes_to_process.push_back(vote);
        crate::jobs::process_votes::start_job_if_required(self);
    }

    pub fn pop_next_vote_to_process(&mut self) -> Option<VoteToProcess> {
        self.votes_to_process.pop_front()
    }

    pub fn votes_to_process_count(&self) -> usize {
        self.votes_to_process.len()
    }
}
