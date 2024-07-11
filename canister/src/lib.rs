use candid::CandidType;
use ic_principal::Principal;
use serde::{Deserialize, Serialize};

mod jobs;
mod lifecycle;
mod memory;
mod state;

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
    pub nns_neuron_id: u64,
    pub wtn_neuron_id: [u8; 32],
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

#[derive(Serialize, Deserialize)]
enum VoteToProcess {
    NnsVote(NnsVote),
    PendingWtnVote(WtnVote),
}

#[derive(Serialize, Deserialize)]
struct NnsVote {
    proposal_id: u64,
    vote: bool,
}

#[derive(Serialize, Deserialize)]
struct WtnVote {
    nns_proposal_id: u64,
    wtn_proposal_id: u64,
    vote: bool,
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
