use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct BootloaderTask {
    #[serde(rename = "type")]
    pub type_: String,
    pub path: PathBuf,
    pub use_poseidon: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BootloaderInput {
    pub tasks: Vec<BootloaderTask>,
    pub single_page: bool,
}

impl Default for BootloaderTask {
    fn default() -> Self {
        Self { type_: "CairoPiePath".to_string(), path: PathBuf::default(), use_poseidon: true }
    }
}

impl Default for BootloaderInput {
    fn default() -> Self {
        Self { tasks: Vec::default(), single_page: true }
    }
}

pub fn write_cairo_pie_zip() {}
