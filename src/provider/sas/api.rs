use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

pub(super) fn get_boarding_pass(booking_reference: &str, last_name: &str) -> BoardingPassDto {
    let client = reqwest::blocking::Client::new();
    let reservation: ReservationDto = client
        .get("https://api2.flysas.com/reservation-service/reservation")
        .query(&[
            ("bookingReference", booking_reference),
            ("names", last_name),
        ])
        .send()
        .expect("could not find booking")
        .json()
        .expect("failed to parse reservation response");
    let reservation_id = reservation.data.legacy.reservation_id;

    client
        .post("https://api2.flysas.com/documents/checkin/boardingpass")
        .json(&GetBoardingPassDto {
            channel: "SK".to_string(),
            delivery_mode: DeliveryMode {
                typ: "JSON".to_string(),
            },
            reservation_id,
            view: "MOW".to_string(),
        })
        .send()
        .expect("could not get boarding pass")
        .json()
        .expect("failed to parse boarding pass response")
}

#[derive(Serialize, Deserialize, Debug)]
pub(super) struct ReservationDto {
    data: ReservationData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReservationData {
    #[serde(rename = "LEGACY")]
    legacy: LegacyReservation,
}

#[derive(Serialize, Deserialize, Debug)]
struct LegacyReservation {
    #[serde(rename = "reservationId")]
    reservation_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GetBoardingPassDto {
    channel: String,
    delivery_mode: DeliveryMode,
    reservation_id: String,
    view: String,
}

#[derive(Serialize, Debug)]
struct DeliveryMode {
    #[serde(rename = "type")]
    typ: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct BoardingPassDto {
    pub document_list: Vec<Document>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Document {
    pub airline_booking_reference: String,
    pub passengers: Vec<Passenger>,
    pub connections: Vec<Connection>,
    pub ancillary_products: Vec<AncillaryProduct>,
    pub documents: Vec<BoardingDocument>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Passenger {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub title: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Connection {
    pub flight_segments: Vec<FlightSegment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct FlightSegment {
    pub id: String,
    pub departure: Gate,
    pub arrival: Gate,
    pub scheduled_departure_date_time_local: NaiveDateTime,
    pub scheduled_arrival_date_time_local: NaiveDateTime,
    pub boarding_time_local: NaiveDateTime,
    pub marketing_carrier: Carrier,
    pub passenger_flight_segments: Vec<PassengerFlightSegment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Gate {
    pub airport_code: String,
    pub airport_name: String,
    #[serde(default)]
    pub gate: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Carrier {
    pub code: String,
    pub flight_number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct PassengerFlightSegment {
    pub associated_passenger: String,
    pub cabin_comments: String,
    pub service_class: String,
    pub boarding_number: String,
    pub boarding_zone: String,
}

// FIXME: probably should be an enum
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct AncillaryProduct {
    pub seat_details: Option<Vec<SeatDetails>>,
    pub associated_passengers: Vec<PassengerLink>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct SeatDetails {
    pub number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct PassengerLink {
    pub id: String,
    pub associated_flight_segments: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct BoardingDocument {
    pub barcode_data: String,
    pub associated_passenger_flight_segments: Vec<String>,
}
