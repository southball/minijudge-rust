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
    pub timeWall: Option<f64>,
    pub memory: Option<i64>,
    pub exitCode: Option<i64>,
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

pub fn print_meta(meta: &Meta) {
    print!("Time: ");
    if let Some(time) = &meta.time { println!("{}", time); } else { println!(); }
    print!("Time-wall: ");
    if let Some(time_wall) = &meta.timeWall { println!("{}", time_wall); } else { println!(); }
    print!("Memory: ");
    if let Some(memory) = &meta.memory { println!("{}", memory); } else { println!(); }
    print!("Verdict: ");
    if let Some(verdict) = &meta.verdict { println!("{}", verdict); } else { println!(); }
    println!();
}

pub fn parse_meta(source: &str) -> Meta {
    let mut meta = Meta {
        time: None,
        timeWall: None,
        memory: None,
        verdict: None,
        exitCode: None,
    };

    let lines: Vec<&str> = source.split('\n').collect();

    for line in lines {
        let find_res = line.find(':');
        if let Some(len) = find_res {
            let key = &line[0..len];
            let value = &line[len + 1..];

            match key {
                "time" => { if let Ok(v) = value.parse::<f64>() { meta.time = Some(v); } },
                "time-wall" => { if let Ok(v) = value.parse::<f64>() { meta.timeWall = Some(v); } },
                "max-rss" => { if let Ok(v) = value.parse::<i64>() { meta.memory = Some(v); } },
                "exitcode" => { if let Ok(v) = value.parse::<i64>() { meta.exitCode = Some(v); } }
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