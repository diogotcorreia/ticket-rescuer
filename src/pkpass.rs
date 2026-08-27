use passes::{
    barcode::{Barcode, BarcodeFormat},
    fields, visual_appearance, Package, PassBuilder, PassConfig,
};

#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

impl From<Color> for Option<visual_appearance::Color> {
    fn from(value: Color) -> Self {
        visual_appearance::Color::new(value.0, value.1, value.2)
    }
}

pub struct PkpassOptions {
    pub organization_name: &'static str,
    pub description: &'static str,
    pub pass_type_identifier: &'static str,

    pub label_color: Color,
    pub foreground_color: Color,
    pub background_color: Color,
}

impl From<&PkpassOptions> for PassConfig {
    fn from(value: &PkpassOptions) -> Self {
        PassConfig {
            organization_name: value.organization_name.into(),
            description: value.description.into(),
            pass_type_identifier: value.pass_type_identifier.into(),
            team_identifier: "AA00AA0A0A".into(),
            serial_number: "ABCDEFG1234567890".into(),
        }
    }
}

impl From<&PkpassOptions> for visual_appearance::VisualAppearance {
    fn from(value: &PkpassOptions) -> Self {
        visual_appearance::VisualAppearance {
            label_color: value.label_color.into(),
            foreground_color: value.foreground_color.into(),
            background_color: value.background_color.into(),
        }
    }
}

pub fn generate_pkpass(boarding_pass: &BoardingPass, options: &PkpassOptions) -> Package {
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

    let pass = PassBuilder::new(options.into())
        .appearance(options.into())
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

#[derive(Debug)]
pub struct BoardingPass {
    pub seat: String,
    pub gate: String,
    pub origin_label: String,
    pub origin_code: String,
    pub destination_label: String,
    pub destination_code: String,
    pub passenger_name: String,
    pub boarding_zone: String,
    pub boarding_number: String,
    pub flight_number: String,
    pub departure_date: String,
    pub boarding_time: String,
    pub class: String,
    pub pnr: String,
    pub service_class: String,
    pub departure: String,
    pub arrival: String,
    pub frequent_flyer: String,
    pub barcode_data: String,
}
