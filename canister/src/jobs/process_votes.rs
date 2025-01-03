use crate::logs::log;
use crate::state::State;
use crate::{state, VoteToProcess, WtnVote};
use candid::CandidType;
use ic_cdk::api::call::CallResult;
use ic_cdk_timers::TimerId;
use ic_principal::Principal;
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
        ic_cdk::spawn(process_vote(vote));
    }
}

async fn process_vote(vote: VoteToProcess) {
    let vote_string = format!("{vote:?}");
    log(format!("Processing vote: {vote_string}"));

    match vote {
        VoteToProcess::NnsVote(pair_id, nns_vote) => {
            let vote_to_process = match state::read(|s| {
                s.get_cached_wtn_proposal_for_nns_proposal(nns_vote.proposal_id)
            }) {
                Some(Some(wtn_proposal_id)) => Some(VoteToProcess::PendingWtnVote(
                    pair_id,
                    WtnVote {
                        nns_proposal_id: nns_vote.proposal_id,
                        wtn_proposal_id,
                        adopt: nns_vote.adopt,
                    },
                )),
                Some(None) => None,
                None => {
                    // Didn't find the WTN proposal in the cache, so call
                    // into WTN canister to retrieve it
                    let canister_id = state::read(|s| s.wtn_protocol_canister_id());
                    let response = get_wtn_proposal_id(canister_id, nns_vote.proposal_id).await;
                    match response {
                        Ok(Ok(wtn_proposal_id)) => {
                            state::mutate(|s| {
                                s.record_wtn_proposal_for_nns_proposal(
                                    nns_vote.proposal_id,
                                    Some(wtn_proposal_id.id),
                                )
                            });
                            Some(VoteToProcess::PendingWtnVote(
                                pair_id,
                                WtnVote {
                                    nns_proposal_id: nns_vote.proposal_id,
                                    wtn_proposal_id: wtn_proposal_id.id,
                                    adopt: nns_vote.adopt,
                                },
                            ))
                        }
                        Ok(Err(latest_processed_nns_proposal_id)) => {
                            if latest_processed_nns_proposal_id.id >= nns_vote.proposal_id {
                                state::mutate(|s| {
                                    s.record_wtn_proposal_for_nns_proposal(
                                        nns_vote.proposal_id,
                                        None,
                                    )
                                });
                                log(format!(
                                    "No WTN proposal found for NNS proposal {}",
                                    nns_vote.proposal_id
                                ));
                                None
                            } else {
                                // The WTN canister hasn't processed this NNS proposal yet, so put the NNS
                                // proposal back in the queue for it to be attempted again shortly
                                log(format!(
                                    "WTN canister has not processed NNS proposal yet. ProposalId: {}. Latest processed: {}",
                                    nns_vote.proposal_id,
                                    latest_processed_nns_proposal_id.id
                                ));
                                Some(VoteToProcess::NnsVote(pair_id, nns_vote))
                            }
                        }
                        Err(error) => {
                            log(format!("Error calling `get_wtn_proposal_id`: {error:?}"));
                            Some(VoteToProcess::NnsVote(pair_id, nns_vote))
                        }
                    }
                }
            };
            if let Some(vote) = vote_to_process {
                state::mutate(|s| s.push_vote_to_process(vote));
            }
        }
        VoteToProcess::PendingWtnVote(pair_id, wtn_vote) => {
            if let Some((canister_id, neuron_id)) = state::read(|s| {
                s.neuron_pairs()
                    .get(&pair_id)
                    .map(|p| (s.wtn_governance_canister_id(), p.wtn_neuron_id()))
            }) {
                let args = ManageNeuronArgs {
                    subaccount: neuron_id.to_vec(),
                    command: Some(Command::RegisterVote(RegisterVote {
                        proposal: Some(ProposalId {
                            id: wtn_vote.wtn_proposal_id,
                        }),
                        vote: if wtn_vote.adopt { 1 } else { 2 },
                    })),
                };
                let response: CallResult<(ManageNeuronResponse,)> =
                    ic_cdk::call(canister_id, "manage_neuron", (&args,)).await;
                state::mutate(|s| match response.map(|r| r.0.command) {
                    Ok(Some(CommandResponse::RegisterVote(_))) => {
                        s.record_wtn_vote_registered(pair_id, wtn_vote);
                    }
                    Ok(Some(CommandResponse::Error(error))) => {
                        log(format!(
                            "Governance canister returned an error: {error:?}. Args: {args:?}"
                        ));
                    }
                    Ok(None) => {
                        log(format!(
                            "Governance canister returned an empty response. Args: {args:?}"
                        ));
                    }
                    Err(error) => {
                        log(format!(
                            "Error calling `manage_neuron`: {error:?}. Args: {args:?}"
                        ));
                        s.push_vote_to_process(VoteToProcess::PendingWtnVote(pair_id, wtn_vote));
                    }
                });
            }
        }
    }

    log(format!("Finished processing vote: {vote_string}"));

    state::read(start_job_if_required);
}

async fn get_wtn_proposal_id(
    canister_id: Principal,
    nns_proposal_id: u64,
) -> CallResult<Result<ProposalId, ProposalId>> {
    let response: CallResult<(Result<ProposalId, ProposalId>,)> =
        ic_cdk::call(canister_id, "get_wtn_proposal_id", (nns_proposal_id,)).await;

    response.map(|r| r.0)
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct ProposalId {
    id: u64,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct ManageNeuronArgs {
    subaccount: Vec<u8>,
    command: Option<Command>,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
enum Command {
    RegisterVote(RegisterVote),
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct RegisterVote {
    proposal: Option<ProposalId>,
    vote: i32,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct ManageNeuronResponse {
    command: Option<CommandResponse>,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
enum CommandResponse {
    Error(GovernanceError),
    RegisterVote(RegisterVoteResponse),
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct RegisterVoteResponse {}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct GovernanceError {
    error_type: i32,
    error_message: String,
}
