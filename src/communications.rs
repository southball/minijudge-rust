use serde::{Serialize, Deserialize};

pub const EVENT_TESTCASE: &'static str = "testcase";
pub const EVENT_SUBMISSION: &'static str = "submission";

#[derive(Serialize, )]
pub struct UpdateEvent<'a, T: Serialize + Deserialize<'a>> {
    pub event_type: String,
    pub event: &'a T,
}
