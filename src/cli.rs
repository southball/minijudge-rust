use crate::languages::Language;
use clap::Clap;
use serde::{Deserialize, Serialize};
use log::LevelFilter;

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

    /// The language code for compiling the checker.
    #[clap(long = "checker-language")]
    pub checker_language: String,

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

/// This is the default ID for testcases when no ID is specified.
fn default_id() -> usize {
    0
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

pub fn detect_language(code: &str, languages_definition: &str) -> Result<Language, ()> {
    let languages_definition = std::fs::File::open(&languages_definition).unwrap();
    let languages: Vec<Language> = serde_yaml::from_reader(languages_definition).unwrap();

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
            3 => LevelFilter::Trace,
            _ => LevelFilter::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_log_level() {
        assert_eq!(calc_log_level(2, false), LevelFilter::Debug);
        assert_eq!(calc_log_level(0, true), LevelFilter::Off);
        
        // Quiet option must override all verbosity options.
        assert_eq!(calc_log_level(2, true), LevelFilter::Off);
    }
}
