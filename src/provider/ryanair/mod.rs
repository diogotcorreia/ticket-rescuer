use inquire::Text;

use crate::{
    pkpass::PkpassOptions,
    provider::{ryanair::api::RyanairApi, ProviderResult, ProviderTrait},
};

mod api;

pub struct RyanairProvider;

impl ProviderTrait for RyanairProvider {
    fn start() -> Vec<ProviderResult> {
        let booking_reference = Text::new("Booking reference")
            .prompt()
            .expect("failed to prompt for booking reference");
        let email = Text::new("Email")
            .prompt()
            .expect("failed to prompt for email");

        let api = RyanairApi::new();
        api.get_boarding_passes(&booking_reference, &email)
            .into_iter()
            .map(|boarding_pass| {
                let api = api.clone();
                let name = format!(
                    "{}_{} {}_{}.pkpass",
                    boarding_pass.pnr,
                    boarding_pass.name.first,
                    boarding_pass.name.last,
                    boarding_pass.departure.date.split('T').next().unwrap()
                );
                let get_bytes = move || api.get_apple_pkpass(&boarding_pass);
                ProviderResult::File {
                    name,
                    contents: Box::new(get_bytes),
                }
            })
            .collect()
    }

    fn get_display_name() -> &'static str {
        "Ryanair"
    }

    fn get_pkpass_options() -> PkpassOptions {
        PkpassOptions::dummy()
    }
}
