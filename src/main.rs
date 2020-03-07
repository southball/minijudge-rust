mod cli;
mod sandbox;
mod judge;
mod languages;

use clap::derive::Clap;
use cli::*;
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc, Mutex};
use languages::Language;
use std::borrow::Borrow;
use simplelog::{CombinedLogger, Config, TermLogger, TerminalMode};

fn judge_thread(
    thread_id: usize,
    thread_sb: sandbox::Sandbox,
    opts: Opts,
    metadata: Metadata,
    judge_output: Arc<Mutex<judge::JudgeOutput>>,
    testcases_stack: Arc<Mutex<Vec<Testcase>>>,
) {
    log::debug!("Thread {} spawned. Sandbox at {}.", thread_id, &thread_sb.path.to_str().unwrap());

    let source_language = cli::detect_language(&opts.language);
    let executable_file = source_language.executable_filename();

    loop {
        let testcase: Option<Testcase> = testcases_stack.lock().unwrap().pop();

        if testcase.is_none() {
            log::debug!("Thread {} finds no test cases and terminates.", thread_id);
            break;
        }

        let Testcase { id, input, output } = testcase.unwrap();
        let mut testcase_output: judge::TestcaseOutput =
            judge_output.lock().unwrap().testcases[id].clone();

        log::debug!("Test {} will be processed by thread {}.", id, thread_id);

        sandbox::copy_into(
            &thread_sb,
            PathBuf::from(&opts.testcases).join(&input).to_str().unwrap(),
            "in.txt").unwrap();

        log::trace!("Test {} executing.", id);
        sandbox::run::<>(
            &thread_sb,
            &*source_language,
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
            &executable_file,
        ).unwrap();
        log::trace!("Test {} executed.", id);

        let meta_file = sandbox::read_file(&thread_sb, "meta.txt").unwrap();
        let meta = judge::parse_meta(&meta_file);

        if let Some(time) = &meta.time { testcase_output.time = *time; }
        if let Some(memory) = &meta.memory { testcase_output.memory = *memory; }
        if let Some(verdict) = &meta.verdict { testcase_output.verdict = verdict.clone(); }

        testcase_output.sandbox_output = meta_file.clone();

        sandbox::copy_into(&thread_sb, PathBuf::from(&opts.testcases).join(&output).to_str().unwrap(), "ans.txt").unwrap();
        let flags = vec!["checker", "in.txt", "out.txt", "ans.txt"];

        log::trace!("Test {} checker executing.", id);
        let _checker_output = sandbox::execute(
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
        log::trace!("Test {} checker executed.", id);

        let checker_output = sandbox::read_file(&thread_sb, "checker.txt").unwrap().trim().to_string();

        let meta = judge::apply_checker_output(&meta, &checker_output);
        testcase_output.checker_output = checker_output.clone();

        if let Some(verdict) = meta.verdict { testcase_output.verdict = verdict.clone(); }

        judge_output.lock().unwrap().testcases[id] = testcase_output.clone();
        log::debug!("Test {} processing completed by thread {}.", id, thread_id);
    }
}

fn compile_source<L: Language + ?Sized>(
    sb: &sandbox::Sandbox,
    language: &L,
    metadata: &Metadata,
    source: &str,
    destination: &str
) -> Result<(), Box<dyn std::error::Error>> {
    sandbox::compile::<L>(
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
        },
        source,
        destination,
    )?;

    Ok(())
}

fn compile_checker<L: Language + ?Sized>(
    sb: &sandbox::Sandbox,
    language: &L,
    metadata: &Metadata,
    source: &str,
    destination: &str
) -> Result<(), Box<dyn std::error::Error>> {
    sandbox::compile::<L>(
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
        },
        source,
        destination,
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let log_level = cli::get_log_level(opts.verbosity, opts.quiet);
    CombinedLogger::init(
        vec![
            TermLogger::new(log_level, Config::default(), TerminalMode::Mixed).unwrap()
        ]
    ).unwrap();

    print_opts(&opts);

    let metadata = read_metadata(&opts.metadata)?;
    print_metadata(&metadata);

    let sandbox_count: i32 = opts.sandboxes;
    assert!(sandbox_count >= 1);

    let mut sandboxes = Vec::new();

    for i in 0..sandbox_count {
        sandboxes.push(sandbox::create_sandbox(i)?);
    }

    let sandbox_primary = &sandboxes[0];

    let source_language = cli::detect_language(&opts.language);
    let source_file = source_language.source_filename();
    let executable_file = source_language.executable_filename();

    sandbox::copy_into(sandbox_primary, &opts.source, &source_file)?;
    compile_source::<>(sandbox_primary, &*source_language, &metadata, &source_file, &executable_file)?;

    sandbox::copy_into(sandbox_primary, &opts.testlib, "./testlib.h")?;
    sandbox::copy_into(sandbox_primary, &opts.checker, "./checker.cpp")?;
    compile_checker::<>(sandbox_primary, languages::LanguageCpp17 {}.borrow(), &metadata, "checker.cpp", "checker")?;

    // Copy the compiled binaries to other sandboxes.
    for sb_sub in sandboxes.iter().skip(1) {
        sandbox::copy_between(&sandbox_primary, &sb_sub, "checker", "checker")?;
        sandbox::copy_between(&sandbox_primary, &sb_sub, &executable_file, &executable_file)?;
    }

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

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    for (thread_id, thread_sb) in sandboxes.iter().enumerate() {
        let thread_id = thread_id;
        let thread_sb = thread_sb.clone();
        let opts = opts.clone();
        let metadata = metadata.clone();
        let judge_output = judge_output.clone();
        let testcases_stack = testcases_stack.clone();

        let thread = thread::spawn(move || {
            judge_thread(
                thread_id,
                thread_sb,
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

    // Output the overall verdict as YAML.
    let output = match &opts.verdict_format[..] {
        "json" => serde_json::to_string(&*judge_output)?,
        "yaml" => serde_yaml::to_string(&*judge_output)?,
        _ => {
            log::warn!("The verdict format is invalid. Defaulting to json.");
            serde_json::to_string(&*judge_output)?
        }
    };

    // Output the verdict to file if provided. Otherwise, output to standard input.
    if &opts.verdict != "" {
        std::fs::write(&opts.verdict, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}
