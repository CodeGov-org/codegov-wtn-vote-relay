use crate::InitArgs;
use ic_principal::Principal;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    nns_governance_canister_id: Principal,
    wtn_governance_canister_id: Principal,
    nns_neuron_id: u64,
    wtn_neuron_id: [u8; 32],
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
    })
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
        }
    }
}
