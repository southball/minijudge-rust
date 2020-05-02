use crate::languages::DynLanguage;
use clap::Clap;
use serde::{Deserialize, Serialize};
use simplelog::LevelFilter;
use std::path::Path;

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

    /// The path to the source code of interactor. If provided, the problem will be treated as
    /// interactive.
    #[clap(long = "interactor")]
    pub interactor: Option<String>,

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

    /// The YAML file containing definition to different languages.
    #[clap(long = "languages-definition")]
    pub languages_definition: String,
}

fn default_id() -> usize {
    0
}

/// This is an error in the user's option passed to the program. The process cannot be continued in this case.
#[derive(Debug, Clone)]
pub struct OptionError {
    pub message: String,
}

impl std::fmt::Display for OptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Option error: {}", self.message)
    }
}

impl std::error::Error for OptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

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
    log::debug!("Sandboxes:  {}", &opts.sandboxes);
    log::debug!("Metadata:   {}", &opts.metadata);
    log::debug!("Language:   {}", &opts.language);
    log::debug!("Source:     {}", &opts.source);
    log::debug!("Checker:    {}", &opts.checker);
    log::debug!(
        "Interactor: {}",
        &opts.interactor.as_ref().unwrap_or(&"None".to_string())
    );
    log::debug!("Testcases:  {}", &opts.testcases);
    log::debug!("Testlib:    {}", &opts.testlib);
    log::debug!(
        "Verdict:    {} ({})",
        &opts.verdict.as_ref().unwrap_or(&"stdout".to_string()),
        &opts.verdict_format
    );
    log::debug!(
        "Socket:     {}",
        &opts.socket.as_ref().unwrap_or(&"None".to_string())
    );
    log::debug!("Lang. Def.: {}", &opts.languages_definition);
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
    let mut metadata: Metadata = match serde_yaml::from_reader(metadata_file) {
        Ok(metadata) => metadata,
        Err(err) => {
            log::error!("Error when parsing metadata: {:?}", err);
            return Err(Box::new(err));
        }
    };

    for (i, testcase) in metadata.testcases.iter_mut().enumerate() {
        testcase.id = i;
    }

    Ok(metadata)
}

pub fn detect_language(code: &str, languages_definition: &str) -> Result<DynLanguage, ()> {
    let languages_definition = std::fs::File::open(&languages_definition).unwrap();
    let languages: Vec<DynLanguage> = serde_yaml::from_reader(languages_definition).unwrap();

    for language in languages {
        if &language.code == code {
            return Ok(language);
        }
    }

    Err(())
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

/// A helper function for `precheck_opts` and `precheck_metadata` returning a 
/// well-formed error if the specified file does not exist.
pub fn assert_exists(path_raw: &str, description: &str) -> Result<(), Box<OptionError>> {
    let path = Path::new(path_raw);

    if path.exists() {
        Ok(())
    } else {
        Err(Box::new(OptionError {
            message: format!("The {} specified at {} does not exist.", description, path_raw)
        }))
    }
}

/// Check that the files specified in the command line options exist.
pub fn precheck_opts(opts: &Opts) -> Result<(), Box<OptionError>> {
    assert_exists(&opts.metadata, "metadata file")?;
    assert_exists(&opts.source, "source file")?;
    assert_exists(&opts.checker, "checker file")?;
    assert_exists(&opts.testcases, "testcases folder")?;
    assert_exists(&opts.testlib, "testlib.h")?;
    assert_exists(&opts.languages_definition, "languages definition file")?;
    
    if let Some(interactor) = &opts.interactor {
        assert_exists(interactor, "interactor file")?;
    }

    Ok(())
}

/// Check that the test files 
pub fn precheck_metadata(opts: &Opts, metadata: &Metadata) -> Result<(), Box<OptionError>> {
    for (testcase_id, testcase) in metadata.testcases.iter().enumerate() {
        let in_path = Path::new(&opts.testcases).join(&testcase.input);
        let out_path = Path::new(&opts.testcases).join(&testcase.output);
        assert_exists(in_path.as_os_str().to_str().unwrap(), &format!("input file for test {}", testcase_id + 1))?;
        assert_exists(out_path.as_os_str().to_str().unwrap(), &format!("output file for test {}", testcase_id + 1))?;
    }

    Ok(())
}
