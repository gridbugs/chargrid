#[macro_use]
extern crate serde;
extern crate prototty_storage;
use prototty_storage::{format, Storage};

const FILE_NAME: &'static str = "state";

#[derive(Serialize, Deserialize)]
struct State {
    cur: u32,
    prev: u32,
}

impl State {
    fn new() -> Self {
        Self { cur: 1, prev: 0 }
    }
    fn next(&mut self) {
        let prev = self.prev;
        self.prev = self.cur;
        self.cur += prev;
    }
}

pub struct App<S: Storage> {
    storage: S,
    state: State,
}

impl<S: Storage> App<S> {
    pub fn new(storage: S) -> Self {
        let state = match storage.load(FILE_NAME, format::Yaml).ok() {
            Some(state) => state,
            None => State::new(),
        };
        Self { state, storage }
    }
    pub fn get(&self) -> u32 {
        self.state.cur
    }
    pub fn next_and_save(&mut self) {
        self.state.next();
        self.storage.store(FILE_NAME, &self.state, format::Yaml).unwrap();
    }
}
