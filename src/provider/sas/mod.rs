use base64::Engine;

use crate::{
    pkpass::{BoardingPass, Color, PkpassOptions},
    provider::{sas::api::BoardingPassDto, Provider},
};

mod api;

pub struct FlysasProvider;

impl Provider for FlysasProvider {
    fn start() -> BoardingPass {
        let stdin = std::io::stdin();
        println!("Booking Reference:");
        let mut booking_reference = String::new();
        stdin.read_line(&mut booking_reference).unwrap();
        println!("Last Name:");
        let mut last_name = String::new();
        stdin.read_line(&mut last_name).unwrap();

        let boarding_pass_dto = api::get_boarding_pass(&booking_reference, &last_name);

        let passenger_flight_segments = get_all_passenger_flight_segments(&boarding_pass_dto);

        // TODO: support more than one boarding pass
        generate_boarding_pass(
            &boarding_pass_dto,
            &passenger_flight_segments[0].0,
            &passenger_flight_segments[0].1,
        )
    }

    fn get_display_name() -> &'static str {
        "Scandinavian Airlines (SAS)"
    }

    fn get_pkpass_options() -> PkpassOptions {
        PkpassOptions {
            organization_name: "SAS",
            description: "SAS Scandinavian Airlines",
            pass_type_identifier: "pass.se.sas.travel",
            label_color: Color(255, 255, 255),
            foreground_color: Color(255, 255, 255),
            background_color: Color(6, 0, 141),
        }
    }
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
