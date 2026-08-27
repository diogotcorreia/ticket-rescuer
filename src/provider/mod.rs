use crate::pkpass::{BoardingPass, PkpassOptions};

pub mod sas;

pub trait Provider {
    fn start() -> BoardingPass;

    fn get_display_name() -> &'static str;

    fn get_pkpass_options() -> PkpassOptions;
}
