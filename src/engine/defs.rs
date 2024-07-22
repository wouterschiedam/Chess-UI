use crate::{comm::CommReport, search::defs::SearchReport};

#[derive(Debug, Clone)]
pub struct Settings {
    pub threads: usize,
    pub quiet: bool,
    pub tt_size: usize,
}

// This enum provides informatin to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq, Debug)]
pub enum Information {
    Comm(CommReport),
    Search(SearchReport),
}

#[derive(PartialEq, Clone)]
pub enum EngineOptionName {
    Hash(String),
    ClearHash,
    Nothing,
}
pub enum UiElement {
    Spin,
    Button,
}
pub struct EngineOption {
    pub name: &'static str,
    pub ui_element: UiElement,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

impl EngineOption {
    pub fn new(
        name: &'static str,
        ui_element: UiElement,
        default: Option<String>,
        min: Option<String>,
        max: Option<String>,
    ) -> Self {
        Self {
            name,
            ui_element,
            default,
            min,
            max,
        }
    }
}

pub struct EngineOptionDefaults;
impl EngineOptionDefaults {
    pub const HASH_DEFAULT: usize = 32;
    pub const HASH_MIN: usize = 0;
    pub const HASH_MAX_64_BIT: usize = 65536;
    pub const HASH_MAX_32_BIT: usize = 2048;
}
