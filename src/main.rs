mod cli;
mod sandbox;
mod judge;
mod languages;

use clap::derive::Clap;
use cli::*;
use std::path::PathBuf;
use crate::languages::LanguageCpp17;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    print_opts(&opts);

    let metadata = read_metadata(&opts.metadata)?;
    print_metadata(&metadata);

    let sandbox_count: i32 = 2;
    assert!(sandbox_count >= 1);

    let mut sandboxes = Vec::new();

    for i in 0..sandbox_count {
        sandboxes.push(sandbox::create_sandbox(i)?);
    }

    let sb = &sandboxes[0];
    let test1 = &metadata.testcases[0];

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
        "checker"
    )?;

    sandbox::copy_into(sb, PathBuf::from(&opts.testcases).join(&test1.input).to_str().unwrap(), "in.txt")?;
    sandbox::run::<LanguageCpp17>(
        sb,
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
    )?;

    println!("Output: {}", sandbox::read_file(sb, "out.txt")?);

    sandbox::copy_into(sb, PathBuf::from(&opts.testcases).join(&test1.output).to_str().unwrap(), "ans.txt")?;
    let flags = vec!["checker", "in.txt", "out.txt", "ans.txt"];
    let output = sandbox::execute(
        sb,
        &sandbox::ExecuteConfig {
            memory_limit: metadata.compile_memory_limit,
            time_limit: metadata.compile_time_limit,
            wall_time_limit: metadata.compile_time_limit,
            meta_file: None,
            full_env: false,
            unlimited_processes: false,
            input_file: None,
            output_file: None,
            error_file: Some("checker.txt"),
        },
        &flags
    )?;

    println!("Checker output: {}", sandbox::read_file(sb, "checker.txt")?);

    Ok(())
}
