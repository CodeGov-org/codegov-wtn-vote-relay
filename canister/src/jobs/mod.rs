use crate::state::State;

mod check_for_new_nns_votes;
pub mod process_votes;

pub fn start_jobs(state: &State) {
    check_for_new_nns_votes::start_job();
    process_votes::start_job_if_required(state);
}
