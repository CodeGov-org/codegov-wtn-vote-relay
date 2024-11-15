use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
    static LOGS: RefCell<VecDeque<String>> = RefCell::default();
}

pub fn init(logs: Vec<String>) {
    LOGS.set(VecDeque::from(logs));
}

pub fn log<S: AsRef<str>>(s: S) {
    let message = s.as_ref();

    ic_cdk::println!("{message}");

    LOGS.with_borrow_mut(|logs| {
        let now = ic_cdk::api::time() / 1_000_000;
        logs.push_back(format!("{now}: {message}"));

        while logs.len() > 5000 {
            logs.pop_front();
        }
    })
}

pub fn logs() -> Vec<String> {
    LOGS.with_borrow(|logs| logs.iter().cloned().collect())
}
