use crate::languages::Language;
use std::clone::Clone;
use std::default::Default;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Clone)]
pub struct Sandbox {
    pub path: PathBuf,
    pub id: i32,
}

impl Sandbox {
    pub fn get_box_path(&self) -> OsString {
        let pathbuf = PathBuf::from(&self.path).join("box");
        pathbuf.into_os_string()
    }
}

#[derive(Clone)]
pub struct ExecuteConfig<'a> {
    pub wall_time_limit: f64,
    pub time_limit: f64,
    pub memory_limit: i64,
    pub meta_file: Option<&'a str>,
    pub input_file: Option<&'a str>,
    pub output_file: Option<&'a str>,
    pub error_file: Option<&'a str>,
    pub full_env: bool,
    pub unlimited_processes: bool,
    pub additional_flags: Option<Vec<&'a str>>,
}

impl Default for ExecuteConfig<'_> {
    fn default() -> ExecuteConfig<'static> {
        ExecuteConfig {
            wall_time_limit: 0.,
            time_limit: 0.,
            memory_limit: 0,
            meta_file: None,
            input_file: None,
            output_file: None,
            error_file: None,
            full_env: false,
            unlimited_processes: false,
            additional_flags: None,
        }
    }
}

impl Sandbox {
    pub fn create(box_id: i32) -> Result<Sandbox, Box<dyn std::error::Error>> {
        // Ensure that there is no sandbox already created.
        Sandbox::cleanup(box_id)?;

        let box_id_flag = format!("--box-id={}", box_id);
        let process = Command::new("isolate")
            .args(&["--cg", "--init", &box_id_flag[..]])
            .output()?;

        let sandbox_path = String::from_utf8_lossy(&process.stdout).trim().to_string();

        log::trace!("Sandbox {} created at {}.", box_id, &sandbox_path);

        Ok(Sandbox {
            id: box_id,
            path: PathBuf::from(sandbox_path),
        })
    }

    pub fn cleanup(box_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        let box_id_flag = format!("--box-id={}", box_id);
        let process = Command::new("isolate")
            .args(&["--cg", "--cleanup", &box_id_flag[..]])
            .output()?;

        assert!(process.status.success(), true);
        log::trace!("Sandbox {} destroyed.", box_id);

        Ok(())
    }

    pub fn execute(
        &self,
        config: &ExecuteConfig,
        command: &[&str],
    ) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let box_id_flag = format!("--box-id={}", self.id);
        let wall_time_flag = format!("--wall-time={}", config.wall_time_limit);
        let time_flag = format!("--time={}", config.time_limit);
        let memory_flag = format!("--mem={}", config.memory_limit);

        let input_flag: String;
        let output_flag: String;
        let error_flag: String;
        let meta_flag: String;

        let mut args: Vec<&str> = Vec::new();

        args.push("--cg");
        args.push(&box_id_flag[..]);
        args.push(&wall_time_flag[..]);
        args.push(&time_flag[..]);
        args.push(&memory_flag[..]);
        args.push("--run");

        if let Some(input_file) = config.input_file {
            input_flag = format!("--stdin={}", input_file);
            args.push(&input_flag);
        }

        if let Some(output_file) = config.output_file {
            output_flag = format!("--stdout={}", output_file);
            args.push(&output_flag);
        }

        if let Some(error_file) = config.error_file {
            error_flag = format!("--stderr={}", error_file);
            args.push(&error_flag);
        }

        if let Some(meta_file) = config.meta_file {
            meta_flag = format!("--meta={}", meta_file);
            args.push(&meta_flag);
        }

        if config.full_env {
            args.push("--full-env");
        }
        if config.unlimited_processes {
            args.push("--processes=0");
        }

        if let Some(additional_flags) = &config.additional_flags {
            for &flag in additional_flags {
                args.push(flag);
            }
        }

        args.push("--");

        for piece in command.iter() {
            args.push(piece);
        }

        let output = Command::new("isolate")
            .current_dir(self.get_box_path())
            .args(&args)
            .output()?;

        log::trace!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        log::trace!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        log::trace!("Status: {}", output.status.code().unwrap());

        Ok(output)
    }

    pub fn compile(
        &self,
        language: &Language,
        config: &ExecuteConfig,
        source: &str,
        destination: &str,
    ) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let flags: Vec<String> = language.compile(source, destination);
        let flags_str: Vec<&str> = flags.iter().map(|s| &s[..]).collect();

        let mut additional_flags = vec![];
        if let Some(flags) = &config.additional_flags {
            for &flag in flags {
                additional_flags.push(flag);
            }
        }
        if let Some(flags) = &language.compile_flags {
            for flag in flags {
                additional_flags.push(flag);
            }
        }
        let additional_flags = if additional_flags.is_empty() {
            None
        } else {
            Some(additional_flags)
        };

        let config = ExecuteConfig {
            additional_flags,
            ..config.clone()
        };

        let output = self.execute(&config, &flags_str)?;

        log::trace!(
            "Compiled {} [{}] from {}.",
            destination,
            language.code,
            source
        );
        log::trace!(
            "  Compile stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        log::trace!(
            "  Compile stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        Ok(output)
    }

    pub fn run(
        &self,
        language: &Language,
        config: &ExecuteConfig,
        executable: &str,
    ) -> Result<Output, Box<dyn std::error::Error>> {
        let flags: Vec<String> = language.execute(executable);
        let flags_str: Vec<&str> = flags.iter().map(|s| &s[..]).collect();

        let mut additional_flags = vec![];
        if let Some(flags) = &config.additional_flags {
            for &flag in flags {
                additional_flags.push(flag);
            }
        }
        if let Some(flags) = &language.execute_flags {
            for flag in flags {
                additional_flags.push(flag);
            }
        }
        let additional_flags = if additional_flags.is_empty() {
            None
        } else {
            Some(additional_flags)
        };

        let config = ExecuteConfig {
            additional_flags,
            ..config.clone()
        };

        let output = self.execute(&config, &flags_str)?;

        log::trace!("Run {} [{}] finished.", executable, language.code);
        log::trace!("  Run stdout: {}", String::from_utf8_lossy(&output.stdout));
        log::trace!("  Run stderr: {}", String::from_utf8_lossy(&output.stderr));

        Ok(output)
    }

    /// Copy a file from outside the sandbox to inside the sandbox.
    /// The destination is relative to the 'box' folder in the sandbox.
    pub fn copy_into(
        &self,
        source: &str,
        destination: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_path = Path::new(source);
        let destination_path = self.path.join("box").join(destination);

        std::fs::copy(&source_path, &destination_path)?;
        log::trace!(
            "Copied (into sandbox) {:?} to {:?}.",
            &source_path,
            &destination_path,
        );

        Ok(())
    }

    /// Copy a file from a sandbox to another or the same sandbox.
    pub fn copy_across_sandbox(
        &self,
        sb_destination: &Sandbox,
        source: &str,
        destination: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source_path = self.path.join("box").join(source);
        let destination_path = sb_destination.path.join("box").join(destination);

        std::fs::copy(&source_path, &destination_path)?;
        log::trace!(
            "Copied (between sandbox) {:?} to {:?}.",
            &source_path,
            &destination_path
        );

        Ok(())
    }

    /// Read a file inside the sandbox.
    /// The source is relative to the 'box' folder in the sandbox.
    pub fn read_file(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        let source_path = self.path.join("box").join(source);
        let file_content = std::fs::read_to_string(source_path.to_str().unwrap())?;

        Ok(file_content)
    }
}
