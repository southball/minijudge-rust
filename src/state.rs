use crate::cli::{Opts, Metadata, Testcase};
use judge_definitions::JudgeOutput;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub opts: Opts,
    pub metadata: Metadata,
    pub judge_output: Arc<Mutex<JudgeOutput>>,
    pub testcases_stack: Arc<Mutex<Vec<Testcase>>>,
    pub socket: Option<Arc<Mutex<zmq::Socket>>>,
}
