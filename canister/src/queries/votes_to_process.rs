use crate::{state, VoteToProcess};
use ic_cdk::query;

#[query]
fn votes_to_process() -> Vec<VoteToProcess> {
    state::read(|s| s.votes_to_process())
}
