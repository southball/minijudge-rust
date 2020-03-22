use serde::{Serialize, Deserialize};
use handlebars::Handlebars;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct DynLanguage {
    pub source_filename: String,
    pub executable_filename: String,
    pub code: String,
    pub compile_command: Vec<String>,
    pub execute_command: Vec<String>,
    pub compile_flags: Option<Vec<String>>,
    pub execute_flags: Option<Vec<String>>,
}

impl DynLanguage {
    pub fn compile(&self, source: &str, destination: &str) -> Vec<String> {
        let template_engine = Handlebars::new();
        let mut map = BTreeMap::new();
        map.insert("source", source);
        map.insert("destination", destination);

        return self.compile_command
            .iter()
            .map(|s| template_engine.render_template(s, &map).unwrap())
            .collect::<>();
    }

    pub fn execute(&self, executable: &str) -> Vec<String> {
        let template_engine = Handlebars::new();
        let mut map = BTreeMap::new();
        map.insert("executable", executable);

        return self.execute_command
            .iter()
            .map(|s| template_engine.render_template(s, &map).unwrap())
            .collect::<>();
    }
}