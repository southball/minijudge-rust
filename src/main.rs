mod cli;
mod sandbox;
mod judge;
mod languages;

use clap::derive::Clap;
use cli::*;
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::languages::LanguageCpp17;
use std::ops::Deref;
use std::convert::TryInto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    print_opts(&opts);

    let metadata = read_metadata(&opts.metadata)?;
    print_metadata(&metadata);

    let sandbox_count: i32 = 16;
    assert!(sandbox_count >= 1);

    let mut sandboxes = Vec::new();

    for i in 0..sandbox_count {
        sandboxes.push(sandbox::create_sandbox(i)?);
    }

    let sb = &sandboxes[0];

    sandbox::copy_into(sb, &opts.source, "./source.cpp")?;
    sandbox::compile::<LanguageCpp17>(
        sb,
        &sandbox::ExecuteConfig {
            memory_limit: metadata.compile_memory_limit,
            time_limit: metadata.compile_time_limit,
            wall_time_limit: metadata.compile_time_limit,
            meta_file: None,
            full_env: true,
            unlimited_processes: true,
            input_file: None,
            output_file: None,
            error_file: None,
        },
        "source.cpp",
        "program",
    )?;

    sandbox::copy_into(sb, &opts.testlib, "./testlib.h")?;
    sandbox::copy_into(sb, &opts.checker, "./checker.cpp")?;
    sandbox::compile::<LanguageCpp17>(
        sb,
        &sandbox::ExecuteConfig {
            memory_limit: metadata.compile_memory_limit,
            time_limit: metadata.compile_time_limit,
            wall_time_limit: metadata.compile_time_limit,
            meta_file: None,
            full_env: true,
            unlimited_processes: true,
            input_file: None,
            output_file: None,
            error_file: None,
        },
        "checker.cpp",
        "checker",
    )?;

    for sb_sub in sandboxes.iter().skip(1) {
        sandbox::copy_between(&sb, &sb_sub, "checker", "checker")?;
        sandbox::copy_between(&sb, &sb_sub, "program", "program")?;
    }

    let testcases: Arc<Mutex<Vec<Testcase>>> = Arc::new(Mutex::new(metadata.testcases.clone()));
    let judge_output: Arc<Mutex<judge::JudgeOutput>> = Arc::new(Mutex::new(judge::JudgeOutput {
        verdict: judge::VERDICT_WJ.to_string(),
        time: 0.0,
        memory: 0,
        testcases: vec![judge::TestcaseOutput {
            verdict: judge::VERDICT_WJ.to_string(),
            time: 0.0,
            memory: 0,
            checker_output: "".to_string(),
            sandbox_output: "".to_string(),
        }; metadata.testcases.len()],
    }));

    testcases.lock().unwrap().reverse();

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    for (thread_id, thread_sb) in sandboxes.into_iter().enumerate() {
        let thread_id = thread_id;
        let testcases = testcases.clone();
        let opts = opts.clone();
        let metadata = metadata.clone();
        let judge_output = judge_output.clone();

        let thread = thread::spawn(move || {
            eprintln!("Thread {} spawned.", thread_id);
            eprintln!("Sandbox path for {} at {}.", thread_id, &thread_sb.path.to_str().unwrap());

            loop {
                eprintln!("Finding testcase...");
                let mut testcase: Option<Testcase> =
                    testcases.lock().unwrap().pop();

                if testcase.is_none() {
                    eprintln!("Thread {} ends.", thread_id);
                    break;
                }

                let Testcase { id, input, output } = testcase.unwrap();
                let mut testcase_output: judge::TestcaseOutput =
                    judge_output.lock().unwrap().testcases[id].clone();

                sandbox::copy_into(
                    &thread_sb,
                    PathBuf::from(&opts.testcases).join(&input).to_str().unwrap(),
                    "in.txt").unwrap();
                sandbox::run::<LanguageCpp17>(
                    &thread_sb,
                    &sandbox::ExecuteConfig {
                        memory_limit: metadata.memory_limit,
                        time_limit: metadata.time_limit,
                        wall_time_limit: metadata.time_limit,
                        meta_file: Some("meta.txt"),
                        full_env: false,
                        unlimited_processes: false,
                        input_file: Some("in.txt"),
                        output_file: Some("out.txt"),
                        error_file: None,
                    },
                    "program",
                ).unwrap();

                let meta_file = sandbox::read_file(&thread_sb, "meta.txt").unwrap();
                let meta = judge::parse_meta(&meta_file);

                if let Some(time) = &meta.time { testcase_output.time = *time; }
                if let Some(memory) = &meta.memory { testcase_output.memory = *memory; }
                if let Some(verdict) = &meta.verdict { testcase_output.verdict = verdict.clone(); }

                testcase_output.sandbox_output = meta_file.clone();

                println!("Test {} output: {}", id, &meta_file);

                sandbox::copy_into(&thread_sb, PathBuf::from(&opts.testcases).join(&output).to_str().unwrap(), "ans.txt").unwrap();
                let flags = vec!["checker", "in.txt", "out.txt", "ans.txt"];
                let output = sandbox::execute(
                    &thread_sb,
                    &sandbox::ExecuteConfig {
                        memory_limit: metadata.checker_memory_limit,
                        time_limit: metadata.checker_time_limit,
                        wall_time_limit: metadata.checker_time_limit,
                        meta_file: None,
                        full_env: false,
                        unlimited_processes: false,
                        input_file: None,
                        output_file: None,
                        error_file: Some("checker.txt"),
                    },
                    &flags,
                ).unwrap();

                let checker_output = sandbox::read_file(&thread_sb, "checker.txt").unwrap().trim().to_string();
                println!("Test {} checker output: {}", id, checker_output);

                let meta = judge::apply_checker_output(&meta, &checker_output);
                testcase_output.checker_output = checker_output.clone();

                if let Some(verdict) = meta.verdict { testcase_output.verdict = verdict.clone(); }

                judge_output.lock().unwrap().testcases[id] = testcase_output.clone();
            }
        });

        threads.push(thread);
    }

    for thread in threads {
        thread.join();
    }

    // Compute overall verdict, time and memory
    let mut judge_output = judge_output.lock().unwrap();
    judge_output.time = judge_output.testcases.iter().map(|t| t.time).fold(-1. / 0., f64::max);
    judge_output.memory = judge_output.testcases.iter().map(|t| t.memory).fold(i64::MIN, i64::max);
    judge_output.verdict =
        match judge_output.testcases.iter()
            .map(|t| &t.verdict)
            .filter(|v| &v[..] != judge::VERDICT_AC)
            .nth(0)
        {
            Some(v) => v.clone(),
            None => judge::VERDICT_AC.to_string()
        };

    let output = serde_yaml::to_string(&*judge_output)?;
    println!("{}", output);

    Ok(())
}
