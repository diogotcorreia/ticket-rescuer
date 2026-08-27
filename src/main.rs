use std::{fs::File, path::Path};

use crate::{
    pkpass::generate_pkpass,
    provider::{sas::FlysasProvider, Provider},
};

mod pkpass;
mod provider;

fn main() {
    // TODO: support more providers
    println!("Provider: {}", FlysasProvider::get_display_name());
    let boarding_pass = FlysasProvider::start();

    let mut package = generate_pkpass(&boarding_pass, &FlysasProvider::get_pkpass_options());

    let pass_name = format!(
        "{}_{}.pkpass",
        &boarding_pass.flight_number, &boarding_pass.passenger_name
    );
    let path = Path::new(&pass_name);
    let file = File::create(path).expect("failed to create file");
    package.write(file).unwrap();
}
