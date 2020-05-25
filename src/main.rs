mod cli;
mod communications;
mod debug;
mod error;
mod judge;
mod languages;
mod precheck;
mod sandbox;
mod state;

use clap::derive::Clap;
use cli::*;
use judge::TestcaseOutput;
use languages::Language;
use sandbox::Sandbox;
use state::AppState;
use std::borrow::Borrow;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    // Derive log level from CLI options and construct logger.
    let log_level = cli::calc_log_level(opts.verbosity, opts.quiet);
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{:5}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Error)
        .level_for("minijudge_rust", log_level)
        .chain(std::io::stdout())
        // .chain(fern::log_file("output.log")?)
        .apply()
        .unwrap();

    debug::debug_opts(&opts);

    let socket: Option<Arc<Mutex<zmq::Socket>>> = if let Some(socket) = &opts.socket {
        let context = zmq::Context::new();
        let responder = context.socket(zmq::PUB).unwrap();
        responder
            .set_sndhwm(1_100_100)
            .expect("Failed setting hwm.");
        responder.bind(socket).expect("Failed binding publisher.");
        Some(Arc::new(Mutex::new(responder)))
    } else {
        None
    };

    let metadata = match read_metadata(&opts.metadata) {
        Ok(metadata) => metadata,
        Err(err) => {
            log::error!("Failed to read metadata.");
            // TODO output SE verdict.
            return Err(err);
        }
    };

    // Output metadata to debug log.
    debug::debug_metadata(&metadata);

    // Check that the environment is valid.
    if let Err(err) = precheck::precheck_env() {
        log::error!("Error when checking environment: {:?}", err);
        return Err(err);
    }

    // Check that the files referred to in opts and metadata all exist.
    if let Err(err) = precheck::precheck_opts(&opts) {
        log::error!("Error when checking command line options: {:?}", err);
        return Err(err);
    }

    if let Err(err) = precheck::precheck_metadata(&opts, &metadata) {
        log::error!("Error when checking metadata: {:?}", err);
        return Err(err);
    }

    log::info!("Options and metadata are checked.");

    // Generate a list of testcases for judge to consume.
    let testcases_stack: Arc<Mutex<Vec<Testcase>>> = Arc::new(Mutex::new(
        metadata
            .testcases
            .clone()
            .into_iter()
            .rev()
            .collect::<Vec<Testcase>>(),
    ));

    let judge_output: Arc<Mutex<judge::JudgeOutput>> = Arc::new(Mutex::new(judge::JudgeOutput {
        verdict: judge::VERDICT_WJ.to_string(),
        time: 0.0,
        memory: 0,
        compile_message: "".into(),
        testcases: vec![
            judge::TestcaseOutput {
                verdict: judge::VERDICT_WJ.to_string(),
                time: 0.0,
                memory: 0,
                checker_output: "".to_string(),
                sandbox_output: "".to_string(),
            };
            metadata.testcases.len()
        ],
    }));

    let sandbox_count: i32 = opts.sandboxes;
    assert!(sandbox_count >= 1);

    let mut sandboxes = Vec::new();

    for i in 0..sandbox_count {
        sandboxes.push(Sandbox::create(i)?);
    }

    let sandbox_primary = &sandboxes[0];

    let source_language = cli::detect_language(&opts.language, &opts.languages_definition).unwrap();
    let source_file = &source_language.source_filename;
    let executable_file = &source_language.executable_filename;

    sandbox_primary.copy_into(&opts.source, &source_file)?;
    let result = match compile_source(
        sandbox_primary,
        &source_language,
        &metadata,
        &source_file,
        &executable_file,
    ) {
        Ok(output) => {
            judge_output.lock().unwrap().compile_message =
                String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(output)
            } else {
                // Compile error
                Err(judge_definitions::verdicts::VERDICT_CE.to_string())
            }
        }
        Err(err) => {
            log::error!("Unexpected error: {:?}", err);
            Err(judge_definitions::verdicts::VERDICT_SE.to_string())
        }
    };

    if let Err(verdict) = result {
        let mut judge_output = judge_output.lock().unwrap();
        judge_output.verdict = verdict.clone();
        for i in 0..judge_output.testcases.len() {
            judge_output.testcases[i] = judge::TestcaseOutput {
                verdict: verdict.clone(),
                ..judge_output.testcases[i].clone()
            };
        }
        flush_verdict(&opts, &judge_output)?;
        return Ok(());
    }

    sandbox_primary.copy_into(&opts.testlib, "./testlib.h")?;
    sandbox_primary.copy_into(&opts.checker, "./checker.cpp")?;

    let result = match compile_checker(
        sandbox_primary,
        &detect_language(&opts.checker_language, &opts.languages_definition).unwrap(),
        &metadata,
        "checker.cpp",
        "checker",
    ) {
        Ok(output) => {
            if output.status.success() {
                Ok(output)
            } else {
                log::error!(
                    "Error when compiling checker:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                Err(judge_definitions::verdicts::VERDICT_SE.to_string())
            }
        }
        Err(err) => {
            log::error!("Unexpected error: {:?}", err);
            Err(judge_definitions::verdicts::VERDICT_SE.to_string())
        }
    };

    if let Err(verdict) = result {
        let mut judge_output = judge_output.lock().unwrap();
        judge_output.verdict = verdict.clone();
        for i in 0..judge_output.testcases.len() {
            judge_output.testcases[i] = judge::TestcaseOutput {
                verdict: verdict.clone(),
                ..judge_output.testcases[i].clone()
            };
        }
        flush_verdict(&opts, &judge_output)?;
        return Ok(());
    }

    // Copy the compiled binaries to other sandboxes.
    for sb_sub in sandboxes.iter().skip(1) {
        sandbox_primary.copy_across_sandbox(&sb_sub, "checker", "checker")?;
        sandbox_primary.copy_across_sandbox(&sb_sub, &executable_file, &executable_file)?;
    }

    let state = Arc::new(AppState {
        opts: opts.clone(),
        metadata: metadata.clone(),
        judge_output: judge_output.clone(),
        testcases_stack: testcases_stack.clone(),
        socket: match &socket {
            Some(s) => Some(s.clone()),
            None => None,
        },
    });

    // Launch the judge threads.
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    for (thread_id, thread_sb) in sandboxes.iter().enumerate() {
        // Clone the variables to be passed into the thread.
        let thread_id = thread_id;
        let thread_sb = thread_sb.clone();
        let state = state.clone();

        let thread = thread::spawn(move || {
            judge_thread(thread_id, thread_sb, state);
        });

        threads.push(thread);
    }

    // Wait for all threads to finish.
    for thread in threads {
        thread.join().unwrap();
    }

    // Compute overall verdict, time and memory
    let mut judge_output = judge_output.lock().unwrap();
    judge::calc_overall_verdict(&mut judge_output);

    flush_verdict(&opts, &*judge_output)?;

    if let Some(socket) = socket {
        let socket = socket.lock().unwrap();
        let submission_json = serde_json::to_string(
            communications::UpdateEvent::<judge::JudgeOutput> {
                event_type: communications::EVENT_SUBMISSION.to_string(),
                event: &*judge_output,
            }
            .borrow(),
        )
        .unwrap();
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

fn judge_thread(thread_id: usize, thread_sb: sandbox::Sandbox, state: Arc<AppState>) {
    log::debug!(
        "Thread {} spawned. Sandbox at {}.",
        thread_id,
        &thread_sb.path.to_str().unwrap()
    );

    let AppState {
        opts,
        metadata,
        judge_output,
        testcases_stack,
        ..
    } = state.as_ref();

    let source_language = cli::detect_language(&opts.language, &opts.languages_definition).unwrap();
    let executable_file = &source_language.executable_filename;

    loop {
        let testcase: Option<Testcase> = state.testcases_stack.lock().unwrap().pop();

        if testcase.is_none() {
            log::debug!("Thread {} finds no test cases and terminates.", thread_id);
            break;
        }

        let Testcase { id, input, output } = testcase.unwrap();
        let mut testcase_output: judge::TestcaseOutput =
            judge_output.lock().unwrap().testcases[id].clone();

        // Output and store the result of the testcase as needed.
        let finalize_testcase = |testcase_output: &mut judge::TestcaseOutput| {
            judge_output.lock().unwrap().testcases[id] = testcase_output.clone();

            let total_testcases = judge_output.lock().unwrap().testcases.len();
            let remaining_testcases = testcases_stack.lock().unwrap().len();
            let processed_testcases = total_testcases - remaining_testcases;

            log::debug!(
                "Progress: {} processed / {} total",
                processed_testcases,
                total_testcases
            );
            log::debug!("Test {} processing completed by thread {}.", id, thread_id);
            log::debug!(
                "Test {}: Verdict = {}, Time = {}, Memory = {}",
                id,
                testcase_output.verdict,
                testcase_output.time,
                testcase_output.memory,
            );

            if let Some(socket) = &state.socket {
                let socket = socket.lock().unwrap();
                let testcase_json = serde_json::to_string(
                    communications::UpdateEvent::<TestcaseOutput> {
                        event_type: communications::EVENT_TESTCASE.to_string(),
                        event: &testcase_output,
                    }
                    .borrow(),
                )
                .unwrap();
                socket.send(&testcase_json, 0).unwrap();
            }
        };

        thread_sb
            .copy_into(
                PathBuf::from(&opts.testcases)
                    .join(&input)
                    .to_str()
                    .unwrap(),
                "in.txt",
            )
            .unwrap();

        log::trace!("Test {} executing.", id);
        let execute_result = thread_sb.run(
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

        if execute_result.is_err() || thread_sb.read_file("meta.txt").is_err() {
            testcase_output.verdict = judge_definitions::verdicts::VERDICT_SE.into();
            finalize_testcase(&mut testcase_output);
            continue;
        }

        let meta_file = thread_sb.read_file("meta.txt").unwrap();
        let meta = judge::parse_meta(&meta_file);

        if let Some(time) = &meta.time {
            testcase_output.time = *time;
        }
        if let Some(memory) = &meta.memory {
            testcase_output.memory = *memory;
        }
        if let Some(verdict) = &meta.verdict {
            testcase_output.verdict = verdict.clone();
        }

        testcase_output.sandbox_output = meta_file.clone();

        if meta.verdict.is_some() {
            finalize_testcase(&mut testcase_output);
            continue;
        }

        thread_sb
            .copy_into(
                PathBuf::from(&opts.testcases)
                    .join(&output)
                    .to_str()
                    .unwrap(),
                "ans.txt",
            )
            .unwrap();
        let flags = vec!["checker", "in.txt", "out.txt", "ans.txt"];

        log::trace!("Test {} checker executing.", id);
        let checker_result = thread_sb.execute(
            &sandbox::ExecuteConfig {
                memory_limit: metadata.checker_memory_limit,
                time_limit: metadata.checker_time_limit,
                wall_time_limit: metadata.checker_time_limit,
                error_file: Some("checker.txt"),
                ..sandbox::ExecuteConfig::default()
            },
            &flags,
        );

        if checker_result.is_err() {
            testcase_output.verdict = judge_definitions::verdicts::VERDICT_SE.into();
            finalize_testcase(&mut testcase_output);
            continue;
        }

        log::trace!("Test {} checker executed.", id);

        let checker_output = thread_sb
            .read_file("checker.txt")
            .unwrap()
            .trim()
            .to_string();

        let meta = judge::apply_checker_output(&meta, &checker_output);
        testcase_output.checker_output = checker_output.clone();

        if let Some(verdict) = meta.verdict {
            testcase_output.verdict = verdict.clone();
        }

        finalize_testcase(&mut testcase_output);
    }
}

/// A helper function for compiling the source program.
fn compile_source(
    sb: &sandbox::Sandbox,
    language: &Language,
    metadata: &Metadata,
    source: &str,
    destination: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    sb.compile(
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
    )
}

/// A helper function for compiling the checker.
fn compile_checker(
    sb: &sandbox::Sandbox,
    language: &Language,
    metadata: &Metadata,
    source: &str,
    destination: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    sb.compile(
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
    )
}
