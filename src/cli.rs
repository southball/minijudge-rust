use clap::Clap;
use serde::{Serialize, Deserialize};

/// MiniJudge-Rust
/// A miniature judge written in Rust.
#[derive(Clap)]
#[clap(version = "0.0-alpha.1", author = "Southball")]
pub struct Opts {
    /// The path to a YAML file containing the metadata, including time limit, memory limit,
    /// test counts, etc.
    #[clap(long = "metadata")]
    pub metadata: String,

    /// The language that the source code was written in.
    #[clap(long = "language")]
    pub language: String,

    /// The path to the file containing source code.
    #[clap(long = "source")]
    pub source: String,

    /// The path to the source code of checker. The source code must be written in C++.
    #[clap(long = "checker")]
    pub checker: String,

    /// The path to be used as the base path of the test cases files.
    #[clap(long = "testcases")]
    pub testcases: String,

    /// The path to testlib.h.
    #[clap(long = "testlib")]
    pub testlib: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Testcase {
    pub input: String,
    pub output: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub problem_name: String,
    pub time_limit: f64,
    pub memory_limit: i32,
    pub compile_time_limit: f64,
    pub compile_memory_limit: i32,
    pub testcases: Vec<Testcase>,
}

pub fn print_opts(opts: &Opts) {
    eprintln!("Metadata:  {}", &opts.metadata);
    eprintln!("Language:  {}", &opts.language);
    eprintln!("Checker:   {}", &opts.checker);
    eprintln!("Source:    {}", &opts.source);
    eprintln!("Testcases: {}", &opts.testcases);
    eprintln!("Testlib:   {}", &opts.testlib);
    eprintln!();
}

pub fn print_metadata(metadata: &Metadata) {
    eprintln!("Problem name:         {}", &metadata.problem_name);
    eprintln!("Time limit:           {}", &metadata.time_limit);
    eprintln!("Memory limit:         {}", &metadata.memory_limit);
    eprintln!("Compile time limit:   {}", &metadata.compile_time_limit);
    eprintln!("Compile memory limit: {}", &metadata.compile_memory_limit);
    eprintln!("Test cases:");
    for (i, testcase) in metadata.testcases.iter().enumerate() {
        eprintln!("  {}: {} -> {}", i + 1, testcase.input, testcase.output);
    }
    eprintln!();
}

pub fn read_metadata(metadata_path: &String) -> Result<Metadata, Box<dyn std::error::Error>> {
    eprintln!("Reading metadata from {}...", &metadata_path);

    let metadata_file = std::fs::File::open(metadata_path)?;
    let metadata: Metadata = serde_yaml::from_reader(metadata_file)?;

    Ok(metadata)
}
