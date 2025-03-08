use std::{fs::File, path::Path};

use base64::Engine;
use chrono::NaiveDateTime;
use passes::{
    barcode::{Barcode, BarcodeFormat},
    fields, visual_appearance, Package, PassBuilder, PassConfig,
};
use serde::{Deserialize, Serialize};

fn main() {
    let stdin = std::io::stdin();
    println!("Booking Reference:");
    let mut booking_reference = String::new();
    stdin.read_line(&mut booking_reference).unwrap();
    println!("Last Name:");
    let mut last_name = String::new();
    stdin.read_line(&mut last_name).unwrap();

    let boarding_pass_dto = get_boarding_pass(&booking_reference, &last_name);

    let passenger_flight_segments = get_all_passenger_flight_segments(&boarding_pass_dto);
    // TODO: support more than one boarding pass
    let pass = generate_boarding_pass(
        &boarding_pass_dto,
        &passenger_flight_segments[0].0,
        &passenger_flight_segments[0].1,
    );
    let mut package = generate_pkpass(&pass);

    let pass_name = format!("{}_{}.pkpass", &pass.flight_number, &pass.passenger_name);
    let path = Path::new(&pass_name);
    let file = File::create(path).expect("failed to create file");
    package.write(file).unwrap();
}

fn get_boarding_pass(booking_reference: &str, last_name: &str) -> BoardingPassDto {
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

fn get_all_passenger_flight_segments(boarding_pass: &BoardingPassDto) -> Vec<(String, String)> {
    boarding_pass
        .document_list
        .iter()
        .flat_map(|doc| {
            doc.connections.iter().flat_map(|conn| {
                conn.flight_segments.iter().flat_map(|seg| {
                    seg.passenger_flight_segments
                        .iter()
                        .map(|pfs| (pfs.associated_passenger.clone(), seg.id.clone()))
                })
            })
        })
        .collect()
}

fn generate_boarding_pass(
    boarding_pass: &BoardingPassDto,
    passenger_id: &str,
    flight_segment_id: &str,
) -> BoardingPass {
    // TODO: support multiple documents?
    let doc = boarding_pass.document_list.first().unwrap();
    let passenger = doc
        .passengers
        .iter()
        .find(|p| p.id == passenger_id)
        .unwrap();
    let flight_segment = doc
        .connections
        .iter()
        .flat_map(|c| &c.flight_segments)
        .find(|fs| fs.id == flight_segment_id)
        .unwrap();
    let passenger_flight_segment = flight_segment
        .passenger_flight_segments
        .iter()
        .find(|pfs| pfs.associated_passenger == passenger_id)
        .unwrap();
    let seat = doc
        .ancillary_products
        .iter()
        .filter(|prod| {
            prod.associated_passengers.iter().any(|ap| {
                ap.id == passenger_id
                    && ap
                        .associated_flight_segments
                        .as_ref()
                        .map(|afs| afs.contains(&flight_segment_id.to_string()))
                        .unwrap_or(false)
            })
        })
        .filter_map(|prod| {
            prod.seat_details
                .as_ref()
                .map(|sd| sd.first().unwrap().number.clone())
        })
        .next()
        .unwrap();
    let barcode_data = doc
        .documents
        .iter()
        .find(|d| {
            d.associated_passenger_flight_segments
                .contains(&format!("{}-{}", passenger_id, flight_segment_id))
        })
        .unwrap()
        .barcode_data
        .clone();
    BoardingPass {
        seat,
        gate: flight_segment.departure.gate.clone(),
        origin_label: flight_segment.departure.airport_name.clone(),
        origin_code: flight_segment.departure.airport_code.clone(),
        destination_label: flight_segment.arrival.airport_name.clone(),
        destination_code: flight_segment.arrival.airport_code.clone(),
        passenger_name: format!(
            "{} {} {}",
            passenger.title, passenger.first_name, passenger.last_name
        ),
        boarding_zone: passenger_flight_segment.boarding_zone.clone(),
        boarding_number: format!("BN{}", passenger_flight_segment.boarding_number.clone()),
        flight_number: format!(
            "{}{}",
            flight_segment.marketing_carrier.code, flight_segment.marketing_carrier.flight_number
        ),
        departure_date: flight_segment
            .scheduled_departure_date_time_local
            .format("%d%b%y")
            .to_string(),
        boarding_time: flight_segment
            .boarding_time_local
            .format("%H:%M")
            .to_string(),
        class: passenger_flight_segment.cabin_comments.clone(),
        pnr: doc.airline_booking_reference.clone(),
        service_class: passenger_flight_segment.service_class.clone(),
        departure: format!(
            "{} {}",
            flight_segment.departure.airport_code,
            flight_segment
                .scheduled_departure_date_time_local
                .format("%H:%M"),
        ),
        arrival: format!(
            "{} {}",
            flight_segment.arrival.airport_code,
            flight_segment
                .scheduled_arrival_date_time_local
                .format("%H:%M"),
        ),
        frequent_flyer: "".to_string(), // TODO?
        barcode_data: String::from_utf8(
            base64::prelude::BASE64_STANDARD
                .decode(barcode_data)
                .unwrap(),
        )
        .unwrap(),
    }
}

fn generate_pkpass(boarding_pass: &BoardingPass) -> Package {
    macro_rules! field {
        ($key: expr, $label: expr, $value: expr) => {
            fields::Content::new(
                $key,
                $value,
                fields::ContentOptions {
                    label: String::from($label).into(),
                    ..Default::default()
                },
            )
        };
    }

    let pass = PassBuilder::new(PassConfig {
        organization_name: "SAS".into(),
        description: "SAS Boarding Pass".into(),
        pass_type_identifier: "pass.se.sas.travel".into(),
        team_identifier: "AA00AA0A0A".into(),
        serial_number: "ABCDEFG1234567890".into(),
    })
    .appearance(visual_appearance::VisualAppearance {
        label_color: visual_appearance::Color::white(),
        foreground_color: visual_appearance::Color::white(),
        background_color: visual_appearance::Color::new(6, 0, 141),
    })
    .fields(
        fields::Type::BoardingPass {
            pass_fields: fields::Fields {
                ..Default::default()
            },
            transit_type: fields::TransitType::Air,
        }
        .add_header_field(field!("seat", "SEAT", &boarding_pass.seat))
        .add_header_field(field!("gate", "GATE", &boarding_pass.gate))
        .add_primary_field(field!(
            "origin",
            &boarding_pass.origin_label,
            &boarding_pass.origin_code
        ))
        .add_primary_field(field!(
            "destination",
            &boarding_pass.destination_label,
            &boarding_pass.destination_code
        ))
        .add_secondary_field(field!(
            "passengerName",
            "NAME",
            &boarding_pass.passenger_name
        ))
        .add_secondary_field(field!(
            "boardingZone",
            "Group",
            &boarding_pass.boarding_zone
        ))
        .add_secondary_field(field!(
            "boardingNumber",
            "SEQ. NO.",
            &boarding_pass.boarding_number
        ))
        .add_auxiliary_field(field!(
            "flightNumber",
            "FLIGHT",
            &boarding_pass.flight_number
        ))
        .add_auxiliary_field(field!(
            "departureDate",
            "DATE",
            &boarding_pass.departure_date
        ))
        .add_auxiliary_field(field!(
            "boardingTime",
            "BOARDING",
            &boarding_pass.boarding_time
        ))
        .add_auxiliary_field(field!("class", "Service Class", &boarding_pass.class))
        .add_back_field(field!(
            "flightNumber",
            "Flight Number",
            &boarding_pass.flight_number
        ))
        .add_back_field(field!("pnr", "PNR", &boarding_pass.pnr))
        .add_back_field(field!(
            "serviceClass",
            "Service Class",
            &boarding_pass.service_class
        ))
        .add_back_field(field!("departure", "Departure", &boarding_pass.departure))
        .add_back_field(field!("arrival", "Arrival", &boarding_pass.arrival))
        .add_back_field(field!(
            "ff",
            "Frequent Flyer",
            &boarding_pass.frequent_flyer
        )),
    )
    .add_barcode(Barcode {
        message: boarding_pass.barcode_data.clone(),
        format: BarcodeFormat::Aztec,
        ..Default::default()
    })
    .build();

    // TODO add icons
    Package::new(pass)
}

#[derive(Serialize, Deserialize, Debug)]
struct ReservationDto {
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
struct BoardingPassDto {
    document_list: Vec<Document>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Document {
    airline_booking_reference: String,
    passengers: Vec<Passenger>,
    connections: Vec<Connection>,
    ancillary_products: Vec<AncillaryProduct>,
    documents: Vec<BoardingDocument>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Passenger {
    id: String,
    first_name: String,
    last_name: String,
    title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Connection {
    flight_segments: Vec<FlightSegment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FlightSegment {
    id: String,
    departure: Gate,
    arrival: Gate,
    scheduled_departure_date_time_local: NaiveDateTime,
    scheduled_arrival_date_time_local: NaiveDateTime,
    boarding_time_local: NaiveDateTime,
    marketing_carrier: Carrier,
    passenger_flight_segments: Vec<PassengerFlightSegment>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Gate {
    airport_code: String,
    airport_name: String,
    gate: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Carrier {
    code: String,
    flight_number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassengerFlightSegment {
    associated_passenger: String,
    cabin_comments: String,
    service_class: String,
    boarding_number: String,
    boarding_zone: String,
}

// FIXME: probably should be an enum
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AncillaryProduct {
    seat_details: Option<Vec<SeatDetails>>,
    associated_passengers: Vec<PassengerLink>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SeatDetails {
    number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PassengerLink {
    id: String,
    associated_flight_segments: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BoardingDocument {
    barcode_data: String,
    associated_passenger_flight_segments: Vec<String>,
}

#[derive(Debug)]
struct BoardingPass {
    seat: String,
    gate: String,
    origin_label: String,
    origin_code: String,
    destination_label: String,
    destination_code: String,
    passenger_name: String,
    boarding_zone: String,
    boarding_number: String,
    flight_number: String,
    departure_date: String,
    boarding_time: String,
    class: String,
    pnr: String,
    service_class: String,
    departure: String,
    arrival: String,
    frequent_flyer: String,
    barcode_data: String,
}
