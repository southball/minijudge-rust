pub trait Language {
    fn source_filename(&self) -> String;
    fn executable_filename(&self) -> String;
    fn compile<'a>(&self, source: &'a str, destination: &'a str) -> Vec<String>;
    fn execute<'a>(&self, executable: &'a str) -> Vec<String>;
}

pub struct LanguageCpp17 {}
pub struct LanguagePython3 {}

impl Language for LanguageCpp17 {
    fn source_filename(&self) -> String { "source.cpp".to_string() }
    fn executable_filename(&self) -> String { "program".to_string() }

    fn compile<'a>(&self, source: &'a str, destination: &'a str) -> Vec<String> {
        return vec![
            "/usr/bin/g++",
            "--std=c++17",
            "-o",
            destination,
            source
        ].iter().map(|s| String::from(*s)).collect();
    }

    fn execute<'a>(&self, executable: &'a str) -> Vec<String> {
        return vec![
            executable,
        ].iter().map(|s| String::from(*s)).collect();
    }
}

impl Language for LanguagePython3 {
    fn source_filename(&self) -> String { "source.py".to_string() }
    fn executable_filename(&self) -> String { "program.py".to_string() }

    fn compile<'a>(&self, source: &'a str, destination: &'a str) -> Vec<String> {
        return vec![
            "/usr/bin/cp",
            source,
            destination,
        ].iter().map(|s| String::from(*s)).collect();
    }

    fn execute<'a>(&self, executable: &'a str) -> Vec<String> {
        return vec![
            "/usr/bin/python3",
            executable
        ].iter().map(|s| String::from(*s)).collect();
    }
}
