use std::path::{Path, PathBuf};
use std::process::Command;
use crate::languages::Language;
use std::ffi::OsString;
use std::clone::Clone;

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
}

pub fn create_sandbox(box_id: i32) -> Result<Sandbox, Box<dyn std::error::Error>> {
    // Ensure that there is no sandbox already created.
    cleanup_sandbox(box_id)?;

    let box_id_flag = format!("--box-id={}", box_id);
    let process = Command::new("isolate")
        .args(&[
            "--cg",
            "--init",
            &box_id_flag[..],
        ])
        .output()?;

    let sandbox_path = String::from_utf8_lossy(&process.stdout).trim().to_string();

    log::trace!("Sandbox {} created at {}.", box_id, &sandbox_path);

    Ok(Sandbox {
        id: box_id,
        path: PathBuf::from(sandbox_path),
    })
}

pub fn cleanup_sandbox(box_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    let box_id_flag = format!("--box-id={}", box_id);
    let process = Command::new("isolate")
        .args(&[
            "--cg",
            "--cleanup",
            &box_id_flag[..],
        ])
        .output()?;

    assert!(process.status.success(), true);
    log::trace!("Sandbox {} destroyed.", box_id);

    Ok(())
}

pub fn execute(sb: &Sandbox, config: &ExecuteConfig, command: &Vec<&str>) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let box_id_flag = format!("--box-id={}", sb.id);
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

    if config.full_env { args.push("--full-env"); }
    if config.unlimited_processes { args.push("--processes=0"); }

    args.push("--");

    for piece in command.iter() {
        args.push(piece);
    }

    let output = Command::new("isolate")
        .current_dir(sb.get_box_path())
        .args(&args)
        .output()?;

    Ok(output)
}

pub fn compile<L: Language + ?Sized>(sb: &Sandbox, language: &L, config: &ExecuteConfig, source: &str, destination: &str) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let flags: Vec<String> = language.compile(source, destination);
    let flags_str: Vec<&str> = flags.iter().map(|s| &s[..]).collect();

    let output = execute(
        &sb,
        &config,
        &flags_str,
    )?;
    log::trace!("Compiled {} [{}] from {}.", destination, language.get_code(), source);

    Ok(output)
}

pub fn run<L: Language + ?Sized>(sb: &Sandbox, language: &L, config: &ExecuteConfig, executable: &str) -> Result<(), Box<dyn std::error::Error>> {
    let flags: Vec<String> = language.execute(executable);
    let flags_str: Vec<&str> = flags.iter().map(|s| &s[..]).collect();
    execute(
        &sb,
        &config,
        &flags_str,
    )?;
    log::trace!("Run {} [{}] finished.", executable, language.get_code());

    Ok(())
}

/// Copy a file from outside the sandbox to inside the sandbox.
/// The destination is relative to the 'box' folder in the sandbox.
pub fn copy_into(sb: &Sandbox, source: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source_pathbuf = Path::new(source);
    let source_path = source_pathbuf.to_str().unwrap();

    let destination_pathbuf = sb.path.join("box").join(destination);
    let destination_path = destination_pathbuf.to_str().unwrap();

    std::fs::copy(source_path, destination_path)?;
    log::trace!("Copied (into sandbox) {} to {}.", source_path, destination_path);

    Ok(())
}

/// Copy a file from a sandbox to another or the same sandbox.
pub fn copy_between(sb_source: &Sandbox, sb_destination: &Sandbox, source: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source_pathbuf = sb_source.path.join("box").join(source);
    let source_path = source_pathbuf.to_str().unwrap();

    let destination_pathbuf = sb_destination.path.join("box").join(destination);
    let destination_path = destination_pathbuf.to_str().unwrap();

    std::fs::copy(source_path, destination_path)?;
    log::trace!("Copied (between sandbox) {} to {}.", source_path, destination_path);

    Ok(())
}

/// Read a file inside the sandbox.
/// The source is relative to the 'box' folder in the sandbox.
pub fn read_file(sb: &Sandbox, source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let source_path = sb.path.join("box").join(source);
    let file_content = std::fs::read_to_string(source_path.to_str().unwrap())?;

    Ok(file_content)
}