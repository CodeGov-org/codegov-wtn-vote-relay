use crate::{InitArgs, NnsVote, VoteToProcess, WtnVote};
use ic_principal::Principal;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    nns_governance_canister_id: Principal,
    wtn_governance_canister_id: Principal,
    nns_neuron_id: u64,
    wtn_neuron_id: [u8; 32],
    latest_seen_nns_vote: Option<u64>, // The proposal Id
    votes_to_process: VecDeque<VoteToProcess>,
    wtn_votes: Vec<WtnVote>,
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
            nns_neuron_id: args.nns_neuron_id,
            wtn_neuron_id: args.wtn_neuron_id,
            latest_seen_nns_vote: None,
            votes_to_process: VecDeque::new(),
            wtn_votes: Vec::new(),
        }
    }

    pub fn nns_governance_canister_id(&self) -> Principal {
        self.nns_governance_canister_id
    }

    pub fn wtn_governance_canister_id(&self) -> Principal {
        self.wtn_governance_canister_id
    }

    pub fn nns_neuron_id(&self) -> u64 {
        self.nns_neuron_id
    }

    pub fn wtn_neuron_id(&self) -> [u8; 32] {
        self.wtn_neuron_id
    }

    pub fn latest_seen_nns_vote(&self) -> Option<u64> {
        self.latest_seen_nns_vote
    }

    pub fn record_nns_vote(&mut self, vote: NnsVote) {
        self.latest_seen_nns_vote = Some(vote.proposal_id);
        self.push_vote_to_process(VoteToProcess::NnsVote(vote));
    }

    pub fn record_wtn_vote_registered(&mut self, vote: WtnVote) {
        self.wtn_votes.push(vote);
    }

    pub fn push_vote_to_process(&mut self, vote: VoteToProcess) {
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
