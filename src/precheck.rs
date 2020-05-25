use crate::cli::{Metadata, Opts};
use crate::error::OptionError;
/// This module contains files to ensure that the files to be used in the
/// judging process specified by the user exists.
use std::path::Path;

/// A helper function for `precheck_opts` and `precheck_metadata` returning a
/// well-formed error if the specified file does not exist.
pub fn assert_exists(path_raw: &str, description: &str) -> Result<(), Box<OptionError>> {
    let path = Path::new(path_raw);

    if path.exists() {
        Ok(())
    } else {
        Err(Box::new(OptionError {
            message: format!(
                "The {} specified at {} does not exist.",
                description, path_raw
            ),
        }))
    }
}

pub fn precheck_env() -> Result<(), Box<OptionError>> {
    let output = std::process::Command::new("which")
        .arg("isolate")
        .output()
        .unwrap();

    if output.status.success() {
        Ok(())
    } else {
        Err(Box::new(OptionError {
            message: "The isolate sandbox is not found in path.".to_string(),
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
        assert_exists(
            in_path.as_os_str().to_str().unwrap(),
            &format!("input file for test {}", testcase_id + 1),
        )?;
        assert_exists(
            out_path.as_os_str().to_str().unwrap(),
            &format!("output file for test {}", testcase_id + 1),
        )?;
    }

    Ok(())
}
