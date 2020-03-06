pub trait Language {
    fn compile<'a>(source: &'a str, destination: &'a str) -> Vec<String>;
    fn execute<'a>(executable: &'a str) -> Vec<String>;
}

pub struct LanguageCpp17 {}
pub struct LanguagePython3 {}

impl Language for LanguageCpp17 {
    fn compile<'a>(source: &'a str, destination: &'a str) -> Vec<String> {
        return vec![
            "/usr/bin/g++",
            "--std=c++17",
            "-o",
            destination,
            source
        ].iter().map(|s| String::from(*s)).collect();
    }

    fn execute<'a>(executable: &'a str) -> Vec<String> {
        return vec![
            executable,
        ].iter().map(|s| String::from(*s)).collect();
    }
}

impl Language for LanguagePython3 {
    fn compile<'a>(source: &'a str, destination: &'a str) -> Vec<String> {
        return vec![
            "/usr/bin/echo"
        ].iter().map(|s| String::from(*s)).collect();
    }

    fn execute<'a>(executable: &'a str) -> Vec<String> {
        return vec![
            "python3",
            executable
        ].iter().map(|s| String::from(*s)).collect();
    }
}
