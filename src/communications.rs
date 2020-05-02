use serde::{Deserialize, Serialize};

pub const EVENT_TESTCASE: &str = "testcase";
pub const EVENT_SUBMISSION: &str = "submission";

#[derive(Serialize)]
pub struct UpdateEvent<'a, T: Serialize + Deserialize<'a>> {
    pub event_type: String,
    pub event: &'a T,
}
