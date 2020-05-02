use crate::cli::{Metadata, Opts};
use crate::judge::Meta;

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

#[allow(dead_code)]
pub fn debug_meta(meta: &Meta) {
    log::debug!(
        "Time: {}",
        if let Some(time) = &meta.time {
            format!("{}", time)
        } else {
            "".to_string()
        }
    );
    log::debug!(
        "Time-wall: {}",
        if let Some(time_wall) = &meta.time_wall {
            format!("{}", time_wall)
        } else {
            "".to_string()
        }
    );
    log::debug!(
        "Memory: {}",
        if let Some(memory) = &meta.memory {
            format!("{}", memory)
        } else {
            "".to_string()
        }
    );
    log::debug!(
        "Verdict: {}",
        if let Some(verdict) = &meta.verdict {
            verdict.to_string()
        } else {
            "".to_string()
        }
    );
}
