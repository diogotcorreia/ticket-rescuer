use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(super) struct RyanairApi {
    client: reqwest::blocking::Client,
}

impl RyanairApi {
    pub(super) fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }

    pub(super) fn get_boarding_passes(
        &self,
        booking_reference: &str,
        email: &str,
    ) -> Vec<BoardingPassDto> {
        self.client
            .post("https://mntappbp.ryanair.com/v1/boardingpass")
            .json(&BoardingPassBodyDto {
                record_locator: booking_reference,
                email,
            })
            .header("client", "trans-rights")
            .send()
            .expect("could not find booking")
            .json()
            .expect("failed to parse reservation response")
    }

    pub(super) fn get_apple_pkpass(&self, boarding_pass: &BoardingPassDto) -> Vec<u8> {
        self.client
            .post("https://mawbp.ryanair.com/v1/downloadpass")
            .json(&AppleBoardingPassBodyDto {
                record_locator: &boarding_pass.pnr,
                sequence_number: boarding_pass.sequence,
                is_infant: boarding_pass.pax_type == "INF",
                departure_station: &boarding_pass.departure.code,
                arrival_station: &boarding_pass.arrival.code,
            })
            .header("client", "trans-rights")
            .send()
            .expect("could not download pkpass")
            .bytes()
            .expect("failed to get pkpass bytes")
            .to_vec()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct BoardingPassDto {
    pub pnr: String,
    pub pax_type: String,
    pub name: PersonName,
    pub departure: AirportAndTime,
    pub arrival: AirportAndTime,
    pub sequence: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct AirportAndTime {
    pub code: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct PersonName {
    pub first: String,
    pub last: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BoardingPassBodyDto<'a> {
    record_locator: &'a str,
    email: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AppleBoardingPassBodyDto<'a> {
    record_locator: &'a str,
    sequence_number: i32,
    is_infant: bool,
    departure_station: &'a str,
    arrival_station: &'a str,
}
