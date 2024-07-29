use candid::CandidType;
use ic_principal::Principal;
use serde::{Deserialize, Serialize};

mod jobs;
mod lifecycle;
mod memory;
mod neuron_pair;
mod state;
mod updates;

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub enum InitOrUpgradeArgs {
    Init(InitArgs),
    Upgrade(UpgradeArgs),
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct InitArgs {
    pub nns_governance_canister_id: Principal,
    pub wtn_governance_canister_id: Principal,
    pub wtn_protocol_canister_id: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Default)]
pub struct UpgradeArgs {}

impl InitOrUpgradeArgs {
    pub fn to_init_args(self) -> InitArgs {
        let InitOrUpgradeArgs::Init(args) = self else {
            panic!("InitOrUpgradeArgs not of type Init");
        };
        args
    }

    pub fn to_upgrade_args(self) -> UpgradeArgs {
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
