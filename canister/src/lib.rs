use candid::CandidType;
use ic_cdk::api::management_canister::main::{CanisterIdRecord, CanisterStatusResponse};
use ic_principal::Principal;
use serde::{Deserialize, Serialize};

mod jobs;
mod lifecycle;
mod logs;
mod memory;
mod neuron_pair;
mod queries;
mod state;
mod updates;

#[derive(CandidType, Serialize, Deserialize, Debug)]
enum InitOrUpgradeArgs {
    Init(InitArgs),
    Upgrade(UpgradeArgs),
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct InitArgs {
    nns_governance_canister_id: Option<Principal>,
    wtn_governance_canister_id: Option<Principal>,
    wtn_protocol_canister_id: Option<Principal>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Default)]
struct UpgradeArgs {}

impl InitOrUpgradeArgs {
    fn into_init_args(self) -> InitArgs {
        let InitOrUpgradeArgs::Init(args) = self else {
            panic!("InitOrUpgradeArgs not of type Init");
        };
        args
    }

    fn into_upgrade_args(self) -> UpgradeArgs {
        let InitOrUpgradeArgs::Upgrade(args) = self else {
            panic!("InitOrUpgradeArgs not of type Upgrade");
        };
        args
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum VoteToProcess {
    NnsVote(u64, NnsVote),
    PendingWtnVote(u64, WtnVote),
}

#[derive(Serialize, Deserialize, Debug)]
struct NnsVote {
    proposal_id: u64,
    adopt: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct WtnVote {
    nns_proposal_id: u64,
    wtn_proposal_id: u64,
    adopt: bool,
}

#[derive(CandidType, Serialize, Deserialize)]
struct RegisterNeuronPairArgs {
    name: String,
    nns_neuron_id: u64,
    wtn_neuron_id: [u8; 32],
}

#[derive(CandidType, Serialize, Deserialize)]
enum RegisterNeuronPairError {
    AlreadyRegistered,
    NotPermittedToVote,
    RegistrationLimitExceeded(u32),
    GovernanceError(i32, String),
    ErrorCallingGovernanceCanister(i32, String),
}

#[derive(CandidType, Serialize, Deserialize)]
struct DeregisterNeuronPairArgs {
    pair_id: u64,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
struct NeuronPairPublic {
    id: u64,
    name: String,
    admin: Principal,
    nns_neuron_id: u64,
    wtn_neuron_id: [u8; 32],
}

#[cfg(test)]
mod generate_candid_file {
    use crate::*;
    use ic_cdk::export_candid;
    use std::env;
    use std::fs::write;
    use std::path::PathBuf;

    #[test]
    fn save_candid() {
        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        export_candid!();
        write(dir.join("can.did"), __export_service()).unwrap()
    }
}
