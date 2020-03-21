use clap::Clap;
use serde::{Serialize, Deserialize};
use crate::languages;
use simplelog::LevelFilter;

/// MiniJudge-Rust
/// A miniature judge written in Rust.
#[derive(Clap, Clone)]
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

    /// The number of sandboxes to be created. The sandbox ID is 0-based.
    #[clap(long = "sandboxes")]
    pub sandboxes: i32,

    /// The format of the verdict to output.
    #[clap(long = "verdict-format", default_value = "json")]
    pub verdict_format: String,

    /// The file to output the verdict to.
    #[clap(long = "verdict")]
    pub verdict: Option<String>,

    /// The level of verbosity.
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbosity: i32,

    /// Whether the log should be suppressed. This option overrides the verbose option.
    #[clap(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Socket to announce updates to. Events are emitted when test cases are completed, and when
    /// the whole submission is judged.
    #[clap(long = "socket")]
    pub socket: Option<String>,
}

fn default_id() -> usize { 0 }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Testcase {
    #[serde(default = "default_id")]
    pub id: usize,
    pub input: String,
    pub output: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub problem_name: String,
    pub time_limit: f64,
    pub memory_limit: i64,
    pub compile_time_limit: f64,
    pub compile_memory_limit: i64,
    pub checker_time_limit: f64,
    pub checker_memory_limit: i64,
    pub testcases: Vec<Testcase>,
}

pub fn debug_opts(opts: &Opts) {
    log::debug!("Sandboxes: {}", &opts.sandboxes);
    log::debug!("Metadata:  {}", &opts.metadata);
    log::debug!("Language:  {}", &opts.language);
    log::debug!("Checker:   {}", &opts.checker);
    log::debug!("Source:    {}", &opts.source);
    log::debug!("Testcases: {}", &opts.testcases);
    log::debug!("Testlib:   {}", &opts.testlib);
}

pub fn debug_metadata(metadata: &Metadata) {
    log::debug!("Problem name:         {}", &metadata.problem_name);
    log::debug!("Time limit:           {}", &metadata.time_limit);
    log::debug!("Memory limit:         {}", &metadata.memory_limit);
    log::debug!("Compile time limit:   {}", &metadata.compile_time_limit);
    log::debug!("Compile memory limit: {}", &metadata.compile_memory_limit);
    log::debug!("Test cases:");
    for (i, testcase) in metadata.testcases.iter().enumerate() {
        log::debug!("  {}: {} -> {}", i + 1, testcase.input, testcase.output);
    }
}

pub fn read_metadata(metadata_path: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    log::debug!("Reading metadata from {}...", &metadata_path);

    let metadata_file = std::fs::File::open(metadata_path)?;
    let mut metadata: Metadata = serde_yaml::from_reader(metadata_file)?;

    for (i, testcase) in metadata.testcases.iter_mut().enumerate() {
        testcase.id = i;
    }

    Ok(metadata)
}

pub fn detect_language(language: &str) -> Result<Box<dyn languages::Language>, ()> {
    match language {
        "cpp17" => Ok(Box::new(languages::LanguageCpp17 {})),
        "python3" => Ok(Box::new(languages::LanguagePython3 {})),
        _ => { Err(()) }
    }
}

pub fn calc_log_level(verbosity: i32, quiet: bool) -> LevelFilter {
    if quiet {
        LevelFilter::Off
    } else {
        match verbosity {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

