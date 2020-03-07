use std::clone::Clone;
use serde::{Serialize, Deserialize};

pub const VERDICT_AC: &'static str = "AC";
pub const VERDICT_WA: &'static str = "WA";
pub const VERDICT_MLE: &'static str = "MLE";
pub const VERDICT_TLE: &'static str = "TLE";
pub const VERDICT_RE: &'static str = "RE";
pub const VERDICT_WJ: &'static str = "WJ";
pub const VERDICT_SE: &'static str = "SE";

#[derive(Clone, Serialize, Deserialize)]
pub struct TestcaseOutput {
    pub verdict: String,
    pub time: f64,
    pub memory: i64,
    pub checker_output: String,
    pub sandbox_output: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JudgeOutput {
    pub verdict: String,
    pub time: f64,
    pub memory: i64,
    pub testcases: Vec<TestcaseOutput>,
}

#[derive(Clone)]
pub struct Meta {
    pub time: Option<f64>,
    pub time_wall: Option<f64>,
    pub memory: Option<i64>,
    pub exit_code: Option<i64>,
    pub verdict: Option<String>,
}

fn map_status(raw_status: &str) -> String {
    match raw_status {
        "RE" => VERDICT_RE.to_string(),
        "SG" => VERDICT_MLE.to_string(),
        "TO" => VERDICT_TLE.to_string(),
        _ => VERDICT_SE.to_string(),
    }
}

// This function is for debug purposes.
#[allow(dead_code)]
pub fn debug_meta(meta: &Meta) {
    log::debug!(
        "Time: {}",
        if let Some(time) = &meta.time { format!("{}", time) } else { "".to_string() }
    );
    log::debug!(
        "Time-wall: {}",
        if let Some(time_wall) = &meta.time_wall { format!("{}", time_wall) } else { "".to_string() }
    );
    log::debug!(
        "Memory: {}",
        if let Some(memory) = &meta.memory { format!("{}", memory) } else { "".to_string() }
    );
    log::debug!(
        "Verdict: {}",
        if let Some(verdict) = &meta.verdict { format!("{}", verdict) } else { "".to_string() }
    );
}

pub fn parse_meta(source: &str) -> Meta {
    let mut meta = Meta {
        time: None,
        time_wall: None,
        memory: None,
        verdict: None,
        exit_code: None,
    };

    let lines: Vec<&str> = source.split('\n').collect();

    for line in lines {
        let find_res = line.find(':');
        if let Some(len) = find_res {
            let key = &line[0..len];
            let value = &line[len + 1..];

            match key {
                "time" => { if let Ok(v) = value.parse::<f64>() { meta.time = Some(v); } },
                "time-wall" => { if let Ok(v) = value.parse::<f64>() { meta.time_wall = Some(v); } },
                "max-rss" => { if let Ok(v) = value.parse::<i64>() { meta.memory = Some(v); } },
                "exitcode" => { if let Ok(v) = value.parse::<i64>() { meta.exit_code = Some(v); } }
                "status" => { meta.verdict = Some(map_status(value)); },
                _ => {},
            };
        }
    }

    meta
}

pub fn apply_checker_output(meta: &Meta, checker_output: &str) -> Meta {
    let mut meta = meta.clone();

    if let None = meta.verdict {
        if checker_output.starts_with("ok") {
            meta.verdict = Some(VERDICT_AC.to_string());
        } else {
            meta.verdict = Some(VERDICT_WA.to_string());
        }
    }

    meta
}

pub fn calc_overall_verdict(judge_output: &mut JudgeOutput) {
    judge_output.time = judge_output.testcases.iter().map(|t| t.time).fold(-1. / 0., f64::max);
    judge_output.memory = judge_output.testcases.iter().map(|t| t.memory).fold(i64::MIN, i64::max);
    judge_output.verdict =
        match judge_output.testcases.iter()
            .map(|t| &t.verdict)
            .filter(|v| &v[..] != VERDICT_AC)
            .nth(0)
        {
            Some(v) => v.clone(),
            None => VERDICT_AC.to_string()
        };
}