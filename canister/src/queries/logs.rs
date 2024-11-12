use ic_cdk::query;

#[query]
fn logs() -> Vec<String> {
    crate::logs::logs()
}
