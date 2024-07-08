use crate::{InitArgs, Vote};
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
    unprocessed_nns_votes: VecDeque<Vote>,
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
            unprocessed_nns_votes: VecDeque::new(),
        }
    }

    pub fn nns_governance_canister_id(&self) -> Principal {
        self.nns_governance_canister_id
    }

    pub fn nns_neuron_id(&self) -> u64 {
        self.nns_neuron_id
    }

    pub fn latest_seen_nns_vote(&self) -> Option<u64> {
        self.latest_seen_nns_vote
    }

    pub fn record_nns_vote(&mut self, vote: Vote) {
        self.latest_seen_nns_vote = Some(vote.proposal_id);
        self.unprocessed_nns_votes.push_back(vote);
    }
}
