mod cli;
mod sandbox;
mod judge;
mod languages;
mod communications;

use clap::derive::Clap;
use cli::*;
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, Mutex};
use languages::DynLanguage;
use std::borrow::Borrow;
use simplelog::{CombinedLogger, Config, TermLogger, TerminalMode};
use judge::TestcaseOutput;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    // Derive log level from CLI options and construct logger.
    let log_level = cli::calc_log_level(opts.verbosity, opts.quiet);
    CombinedLogger::init(
        vec![
            TermLogger::new(log_level, Config::default(), TerminalMode::Mixed).unwrap()
        ]
    ).unwrap();

    debug_opts(&opts);

    let socket: Option<Arc<Mutex<zmq::Socket>>> = if let Some(socket) = &opts.socket {
        let context = zmq::Context::new();
        let responder = context.socket(zmq::PUB).unwrap();
        responder.set_sndhwm(1_100_100).expect("Failed setting hwm.");
        responder.bind(socket).expect("Failed binding publisher.");
        Some(Arc::new(Mutex::new(responder)))
    } else {
        None
    };

    let metadata = read_metadata(&opts.metadata)?;
    debug_metadata(&metadata);

    let testcases_stack: Arc<Mutex<Vec<Testcase>>> = Arc::new(Mutex::new(
        metadata.testcases.clone().into_iter().rev().collect::<Vec<Testcase>>()
    ));

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

    let sandbox_count: i32 = opts.sandboxes;
    assert!(sandbox_count >= 1);

    let mut sandboxes = Vec::new();

    for i in 0..sandbox_count {
        sandboxes.push(sandbox::create_sandbox(i)?);
    }

    let sandbox_primary = &sandboxes[0];

    let source_language = cli::detect_language(&opts.language, &opts.languages_definition).unwrap();
    let source_file = &source_language.source_filename;
    let executable_file = &source_language.executable_filename;

    sandbox::copy_into(sandbox_primary, &opts.source, &source_file)?;
    let result = compile_source::<>(sandbox_primary, &source_language, &metadata, &source_file, &executable_file);

    if let Err(_) = result {
        let mut judge_output = judge_output.lock().unwrap();
        judge_output.verdict = judge_definitions::verdicts::VERDICT_CE.into();
        for i in 0..judge_output.testcases.len() {
            judge_output.testcases[i] = judge::TestcaseOutput {
                verdict: judge_definitions::verdicts::VERDICT_CE.into(),
                ..judge_output.testcases[i].clone()
            };
        }
        flush_verdict(&opts, &judge_output)?;
        return Ok(());
    }

    sandbox::copy_into(sandbox_primary, &opts.testlib, "./testlib.h")?;
    sandbox::copy_into(sandbox_primary, &opts.checker, "./checker.cpp")?;
    let result = compile_checker::<>(sandbox_primary, &detect_language("cpp17", &opts.languages_definition).unwrap(), &metadata, "checker.cpp", "checker");

    if let Err(err) = result {
        println!("{:?}", err);

        let mut judge_output = judge_output.lock().unwrap();
        judge_output.verdict = judge_definitions::verdicts::VERDICT_SE.into();
        for i in 0..judge_output.testcases.len() {
            judge_output.testcases[i] = judge::TestcaseOutput {
                verdict: judge_definitions::verdicts::VERDICT_SE.into(),
                ..judge_output.testcases[i].clone()
            };
        }
        flush_verdict(&opts, &judge_output)?;
        return Ok(());
    }

    // Copy the compiled binaries to other sandboxes.
    for sb_sub in sandboxes.iter().skip(1) {
        sandbox::copy_between(&sandbox_primary, &sb_sub, "checker", "checker")?;
        sandbox::copy_between(&sandbox_primary, &sb_sub, &executable_file, &executable_file)?;
    }

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    for (thread_id, thread_sb) in sandboxes.iter().enumerate() {
        let thread_id = thread_id;
        let thread_sb = thread_sb.clone();
        let opts = opts.clone();
        let metadata = metadata.clone();
        let judge_output = judge_output.clone();
        let testcases_stack = testcases_stack.clone();
        let socket = match &socket {
            Some(s) => Some(s.clone()),
            None => None
        };

        let thread = thread::spawn(move || {
            judge_thread(
                thread_id,
                thread_sb,
                socket,
                opts,
                metadata,
                judge_output,
                testcases_stack,
            );
        });

        threads.push(thread);
    }

    // Wait for all threads to finish.
    for thread in threads { thread.join().unwrap(); }

    // Compute overall verdict, time and memory
    let mut judge_output = judge_output.lock().unwrap();
    judge::calc_overall_verdict(&mut judge_output);

    flush_verdict(&opts, &*judge_output)?;

    if let Some(socket) = socket {
        let socket = socket.lock().unwrap();
        let submission_json = serde_json::to_string(communications::UpdateEvent::<judge::JudgeOutput> {
            event_type: communications::EVENT_SUBMISSION.to_string(),
            event: &*judge_output
        }.borrow()).unwrap();
        socket.send(&submission_json, 0).unwrap();
    }

    Ok(())
}

fn flush_verdict(
    opts: &Opts,
    judge_output: &judge::JudgeOutput,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate the overall verdict.
    let output = match &opts.verdict_format[..] {
        "json" => serde_json::to_string(&*judge_output)?,
        "yaml" => serde_yaml::to_string(&*judge_output)?,
        _ => {
            log::warn!("The verdict format is invalid. Defaulting to json.");
            serde_json::to_string(&*judge_output)?
        }
    };

    // Output the verdict to file if provided. Otherwise, output to standard input.
    if let Some(verdict_file) = &opts.verdict {
        std::fs::write(verdict_file, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}

fn judge_thread(
    thread_id: usize,
    thread_sb: sandbox::Sandbox,
    socket: Option<Arc<Mutex<zmq::Socket>>>,
    opts: Opts,
    metadata: Metadata,
    judge_output: Arc<Mutex<judge::JudgeOutput>>,
    testcases_stack: Arc<Mutex<Vec<Testcase>>>,
) {
    log::debug!("Thread {} spawned. Sandbox at {}.", thread_id, &thread_sb.path.to_str().unwrap());

    let source_language = cli::detect_language(&opts.language, &opts.languages_definition).unwrap();
    let executable_file = &source_language.executable_filename;

    loop {
        let testcase: Option<Testcase> = testcases_stack.lock().unwrap().pop();

        if testcase.is_none() {
            log::debug!("Thread {} finds no test cases and terminates.", thread_id);
            break;
        }

        let Testcase { id, input, output } = testcase.unwrap();
        let mut testcase_output: judge::TestcaseOutput =
            judge_output.lock().unwrap().testcases[id].clone();

        /// Output and store the result of the testcase as needed.
        let finalize_testcase = |testcase_output: &mut judge::TestcaseOutput| {
            judge_output.lock().unwrap().testcases[id] = testcase_output.clone();
            log::debug!("Test {} processing completed by thread {}.", id, thread_id);
            log::debug!(
                "Test {}: Verdict = {}, Time = {}, Memory = {}",
                id,
                testcase_output.verdict,
                testcase_output.time,
                testcase_output.memory,
            );

            if let Some(socket) = &socket {
                let socket = socket.lock().unwrap();
                let testcase_json = serde_json::to_string(communications::UpdateEvent::<TestcaseOutput> {
                    event_type: communications::EVENT_TESTCASE.to_string(),
                    event: &testcase_output,
                }.borrow()).unwrap();
                socket.send(&testcase_json, 0).unwrap();
            }
        };

        log::debug!("Test {} will be processed by thread {}.", id, thread_id);

        sandbox::copy_into(
            &thread_sb,
            PathBuf::from(&opts.testcases).join(&input).to_str().unwrap(),
            "in.txt").unwrap();

        log::trace!("Test {} executing.", id);
        let execute_result = sandbox::run(
            &thread_sb,
            &source_language,
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
                ..sandbox::ExecuteConfig::default()
            },
            &executable_file,
        );
        log::trace!("Test {} executed.", id);

        if execute_result.is_err() || !execute_result.unwrap().status.success() || sandbox::read_file(&thread_sb, "meta.txt").is_err() {
            testcase_output.verdict = judge_definitions::verdicts::VERDICT_SE.into();
            finalize_testcase(&mut testcase_output);
            continue;
        }

        let meta_file = sandbox::read_file(&thread_sb, "meta.txt").unwrap();
        let meta = judge::parse_meta(&meta_file);

        if let Some(time) = &meta.time { testcase_output.time = *time; }
        if let Some(memory) = &meta.memory { testcase_output.memory = *memory; }
        if let Some(verdict) = &meta.verdict { testcase_output.verdict = verdict.clone(); }

        testcase_output.sandbox_output = meta_file.clone();

        sandbox::copy_into(&thread_sb, PathBuf::from(&opts.testcases).join(&output).to_str().unwrap(), "ans.txt").unwrap();
        let flags = vec!["checker", "in.txt", "out.txt", "ans.txt"];

        log::trace!("Test {} checker executing.", id);
        let checker_result = sandbox::execute(
            &thread_sb,
            &sandbox::ExecuteConfig {
                memory_limit: metadata.checker_memory_limit,
                time_limit: metadata.checker_time_limit,
                wall_time_limit: metadata.checker_time_limit,
                error_file: Some("checker.txt"),
                ..sandbox::ExecuteConfig::default()
            },
            &flags,
        );

        if checker_result.is_err() || !checker_result.unwrap().status.success() {
            testcase_output.verdict = judge_definitions::verdicts::VERDICT_SE.into();
            finalize_testcase(&mut testcase_output);
            continue;
        }

        log::trace!("Test {} checker executed.", id);

        let checker_output = sandbox::read_file(&thread_sb, "checker.txt").unwrap().trim().to_string();

        let meta = judge::apply_checker_output(&meta, &checker_output);
        testcase_output.checker_output = checker_output.clone();

        if let Some(verdict) = meta.verdict { testcase_output.verdict = verdict.clone(); }

        finalize_testcase(&mut testcase_output);
    }
}

fn compile_source(
    sb: &sandbox::Sandbox,
    language: &DynLanguage,
    metadata: &Metadata,
    source: &str,
    destination: &str
) -> Result<(), Box<dyn std::error::Error>> {
    sandbox::compile(
        sb,
        language,
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
            ..sandbox::ExecuteConfig::default()
        },
        source,
        destination,
    )?;

    Ok(())
}

fn compile_checker(
    sb: &sandbox::Sandbox,
    language: &DynLanguage,
    metadata: &Metadata,
    source: &str,
    destination: &str
) -> Result<(), Box<dyn std::error::Error>> {
    sandbox::compile(
        sb,
        language,
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
            additional_flags: Some(vec!["--full-env"]),
            ..sandbox::ExecuteConfig::default()
        },
        source,
        destination,
    )?;

    Ok(())
}
