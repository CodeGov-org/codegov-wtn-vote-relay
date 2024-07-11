use crate::{state, NnsVote};
use candid::CandidType;
use ic_cdk::api::call::CallResult;
use ic_principal::Principal;
use serde::Deserialize;
use std::time::Duration;

const CHECK_FOR_NEW_NNS_VOTES_INTERVAL: Duration = Duration::from_secs(120);

pub fn start_job() {
    ic_cdk_timers::set_timer_interval(CHECK_FOR_NEW_NNS_VOTES_INTERVAL, || ic_cdk::spawn(run()));
}

async fn run() {
    ic_cdk::println!("Checking for new NNS votes");

    let (nns_governance_canister_id, nns_neuron_id) =
        state::read(|s| (s.nns_governance_canister_id(), s.nns_neuron_id()));

    match get_neuron_info(nns_governance_canister_id, nns_neuron_id).await {
        Ok(Ok(neuron)) => state::mutate(|s| {
            let latest_seen_vote = s.latest_seen_nns_vote();
            for vote in neuron
                .recent_ballots
                .into_iter()
                .filter_map(|b| NnsVote::try_from(b).ok())
                .take_while(|v| Some(v.proposal_id) != latest_seen_vote)
            {
                s.record_nns_vote(vote);
            }
            ic_cdk::println!("Check for new NNS votes completed successfully");
        }),
        error => {
            ic_cdk::eprintln!("Error calling `get_neuron_info`: {error:?}")
        }
    }
}

async fn get_neuron_info(
    nns_governance_canister_id: Principal,
    nns_neuron_id: u64,
) -> CallResult<Result<NeuronInfo, GovernanceError>> {
    let response: CallResult<(Result<NeuronInfo, GovernanceError>,)> = ic_cdk::call(
        nns_governance_canister_id,
        "get_neuron_info",
        (nns_neuron_id,),
    )
    .await;

    response.map(|r| r.0)
}

#[derive(CandidType, Deserialize, Debug)]
struct NeuronInfo {
    recent_ballots: Vec<BallotInfo>,
}

#[derive(CandidType, Deserialize, Debug)]
struct BallotInfo {
    vote: i32,
    proposal_id: Option<ProposalId>,
}

#[derive(CandidType, Deserialize, Debug)]
struct ProposalId {
    id: u64,
}

#[derive(CandidType, Deserialize, Debug)]
struct GovernanceError {
    error_message: String,
    error_type: i32,
}

impl TryFrom<BallotInfo> for NnsVote {
    type Error = ();

    fn try_from(value: BallotInfo) -> Result<Self, Self::Error> {
        if let Some(proposal_id) = value.proposal_id {
            Ok(NnsVote {
                proposal_id: proposal_id.id,
                adopt: value.vote == 1,
            })
        } else {
            Err(())
        }
    }
}
