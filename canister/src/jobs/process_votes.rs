use crate::state::State;
use crate::{state, VoteToProcess, WtnVote};
use candid::CandidType;
use ic_cdk::api::call::CallResult;
use ic_cdk_timers::TimerId;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::time::Duration;

thread_local! {
    static TIMER_ID: Cell<Option<TimerId>> = Cell::default();
}

pub(crate) fn start_job_if_required(state: &State) {
    if TIMER_ID.get().is_none() && state.votes_to_process_count() > 0 {
        let timer_id = ic_cdk_timers::set_timer(Duration::ZERO, run);
        TIMER_ID.set(Some(timer_id));
    }
}

fn run() {
    TIMER_ID.set(None);

    if let Some(vote) = state::mutate(|s| s.pop_next_vote_to_process()) {
        ic_cdk::println!("Processing vote: {vote:?}");
        ic_cdk::spawn(process_vote(vote));
    }
}

async fn process_vote(vote: VoteToProcess) {
    match vote {
        VoteToProcess::NnsVote(pair_id, nns_vote) => {
            let canister_id = state::read(|s| s.wtn_protocol_canister_id());
            let response: CallResult<(Option<u64>,)> =
                ic_cdk::call(canister_id, "get_wtn_proposal_id", (nns_vote.proposal_id,)).await;
            let vote_to_process = match response.map(|r| r.0) {
                Ok(Some(wtn_proposal_id)) => Some(VoteToProcess::PendingWtnVote(
                    pair_id,
                    WtnVote {
                        nns_proposal_id: nns_vote.proposal_id,
                        wtn_proposal_id,
                        adopt: nns_vote.adopt,
                    },
                )),
                Ok(None) => {
                    ic_cdk::println!(
                        "No WTN proposal found for NNS proposal {}",
                        nns_vote.proposal_id
                    );
                    None
                }
                Err(error) => {
                    ic_cdk::eprintln!("Error calling `get_wtn_proposal_id`: {error:?}");
                    Some(VoteToProcess::NnsVote(pair_id, nns_vote))
                }
            };
            if let Some(vote) = vote_to_process {
                state::mutate(|s| s.push_vote_to_process(vote));
            }
        }
        VoteToProcess::PendingWtnVote(pair_id, wtn_vote) => {
            let Some((canister_id, neuron_id)) = state::read(|s| {
                s.neuron_pair(pair_id)
                    .map(|p| (s.wtn_governance_canister_id(), p.wtn_neuron_id()))
            }) else {
                return;
            };
            let args = ManageNeuronArgs {
                subaccount: neuron_id.to_vec(),
                command: Some(Command::RegisterVote(RegisterVote {
                    proposal: Some(ProposalId {
                        id: wtn_vote.wtn_proposal_id,
                    }),
                    vote: if wtn_vote.adopt { 1 } else { 2 },
                })),
            };
            let response: CallResult<()> =
                ic_cdk::call(canister_id, "manage_neuron", (args,)).await;
            state::mutate(|s| {
                if let Err(error) = response {
                    ic_cdk::eprintln!("Error calling `manage_neuron`: {error:?}");
                    s.push_vote_to_process(VoteToProcess::PendingWtnVote(pair_id, wtn_vote));
                } else {
                    s.record_wtn_vote_registered(pair_id, wtn_vote);
                }
            });
        }
    }

    state::read(start_job_if_required);
}

#[derive(CandidType, Serialize, Deserialize)]
struct ManageNeuronArgs {
    subaccount: Vec<u8>,
    command: Option<Command>,
}

#[derive(CandidType, Serialize, Deserialize)]
enum Command {
    RegisterVote(RegisterVote),
}

#[derive(CandidType, Serialize, Deserialize)]
struct RegisterVote {
    proposal: Option<ProposalId>,
    vote: i32,
}

#[derive(CandidType, Serialize, Deserialize)]
struct ProposalId {
    id: u64,
}
