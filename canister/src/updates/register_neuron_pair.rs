use crate::{state, RegisterNeuronPairArgs, RegisterNeuronPairError};
use candid::CandidType;
use ic_cdk::api::call::CallResult;
use ic_cdk::update;
use ic_principal::Principal;
use serde::{Deserialize, Serialize};

const REGISTER_VOTE_PERMISSION: i32 = 4;

#[update]
async fn register_neuron_pair(
    args: RegisterNeuronPairArgs,
) -> Result<u64, RegisterNeuronPairError> {
    let caller = ic_cdk::caller();
    let wtn_governance_canister = state::read(|s| s.wtn_governance_canister_id());

    match call_get_neuron(wtn_governance_canister, args.wtn_neuron_id).await {
        Ok(response) => match response.0.result.unwrap() {
            GetNeuronResult::Neuron(neuron) => {
                let this_canister_id = ic_cdk::id();
                if !neuron.permissions.iter().any(|p| {
                    p.principal == Some(this_canister_id)
                        && p.permission_type.contains(&REGISTER_VOTE_PERMISSION)
                }) {
                    return Err(RegisterNeuronPairError::NotPermittedToVote);
                }
            }
            GetNeuronResult::Error(error) => {
                return Err(RegisterNeuronPairError::GovernanceError(
                    error.error_type,
                    error.error_message,
                ));
            }
        },
        Err((code, msg)) => {
            return Err(RegisterNeuronPairError::ErrorCallingGovernanceCanister(
                code as i32,
                msg,
            ))
        }
    };

    if let Some(pair_id) = state::mutate(|s| {
        s.register_neuron_pair(caller, args.name, args.nns_neuron_id, args.wtn_neuron_id)
    }) {
        Ok(pair_id)
    } else {
        Err(RegisterNeuronPairError::AlreadyRegistered)
    }
}

async fn call_get_neuron(
    governance_canister: Principal,
    neuron_id: [u8; 32],
) -> CallResult<(GetNeuronResponse,)> {
    let get_neuron_args = GetNeuronArgs {
        neuron_id: NeuronId { id: neuron_id },
    };
    ic_cdk::call(governance_canister, "get_neuron", (get_neuron_args,)).await
}

#[derive(CandidType, Serialize)]
struct GetNeuronArgs {
    neuron_id: NeuronId,
}

#[derive(CandidType, Serialize)]
struct NeuronId {
    id: [u8; 32],
}

#[derive(CandidType, Deserialize)]
struct GetNeuronResponse {
    result: Option<GetNeuronResult>,
}

#[derive(CandidType, Deserialize)]
enum GetNeuronResult {
    Neuron(Neuron),
    Error(GovernanceError),
}

#[derive(CandidType, Deserialize)]
struct Neuron {
    permissions: Vec<NeuronPermission>,
}

#[derive(CandidType, Deserialize)]
struct NeuronPermission {
    principal: Option<Principal>,
    permission_type: Vec<i32>,
}

#[derive(CandidType, Deserialize)]
struct GovernanceError {
    error_type: i32,
    error_message: String,
}
