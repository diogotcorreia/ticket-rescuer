use std::{fs::File, io::Write, path::Path};

use crate::{
    pkpass::generate_pkpass,
    provider::{prompt_provider, ProviderResult},
};

mod pkpass;
mod provider;

fn main() {
    let provider = prompt_provider();
    let results = provider.start();

    for result in results {
        match result {
            ProviderResult::File { name, contents } => {
                let path = Path::new(&name);
                let mut file = File::create(path).expect("failed to create file");
                let bytes = contents();
                file.write_all(&bytes).expect("failed to write file");
                println!("Saved file to {path:#?}")
            }
            ProviderResult::BoardingPass(boarding_pass) => {
                let name = boarding_pass.get_file_name();
                let mut package = generate_pkpass(&boarding_pass, &provider.get_pkpass_options());
                let path = Path::new(&name);
                let file = File::create(path).expect("failed to create file");
                package.write(file).expect("failed to write file");
                println!("Saved file to {path:#?}")
            }
        }
    }
}
