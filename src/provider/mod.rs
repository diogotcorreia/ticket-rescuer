use std::fmt::Display;

use inquire::Select;

use crate::{
    pkpass::{BoardingPass, PkpassOptions},
    provider::{ryanair::RyanairProvider, sas::FlysasProvider},
};

pub mod ryanair;
pub mod sas;

trait ProviderTrait {
    fn start() -> Vec<ProviderResult>;

    fn get_display_name() -> &'static str;

    fn get_pkpass_options() -> PkpassOptions;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Ryanair,
    Flysas,
}

// TODO proc-macros
impl Provider {
    const ALL: &'static [Self] = &[Self::Ryanair, Self::Flysas];

    pub fn start(&self) -> Vec<ProviderResult> {
        match self {
            Self::Ryanair => RyanairProvider::start(),
            Self::Flysas => FlysasProvider::start(),
        }
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            Self::Ryanair => RyanairProvider::get_display_name(),
            Self::Flysas => FlysasProvider::get_display_name(),
        }
    }

    pub fn get_pkpass_options(&self) -> PkpassOptions {
        match self {
            Self::Ryanair => RyanairProvider::get_pkpass_options(),
            Self::Flysas => FlysasProvider::get_pkpass_options(),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_display_name())
    }
}

#[allow(clippy::large_enum_variant)]
pub enum ProviderResult {
    File {
        name: String,
        contents: Box<dyn FnOnce() -> Vec<u8>>,
    },
    BoardingPass(BoardingPass),
}

/// Prompt the user for which provider they want
pub fn prompt_provider() -> Provider {
    Select::new("Select provider", Provider::ALL.to_vec())
        .prompt()
        .expect("failed to prompt for provider")
}
