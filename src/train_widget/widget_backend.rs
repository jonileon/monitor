use std::env::{self};
use crate::train_widget::widget::{Departure, Station};
use chrono::{DateTime, Timelike, Utc};
use chrono_tz::Europe::Berlin;
use trias_rs::requests::stop_event_request::{StopEventResult, get_trips_for_location};

pub fn parse_departures_for_widget() -> Station {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let station_ref = env::var("TRAIN_STATION").expect("No train station found in .env");
    let url = env::var("TRAIN_URL").expect("No url for trias api found.");
    let response = rt.block_on(async {get_trips_for_location(&station_ref, 10, &url).await.unwrap()});
    trias_response_to_station(response)
}

fn trias_response_to_station( responses: Vec<StopEventResult> ) -> Station {
    let mut departures = Vec::new();
    if responses.is_empty() {
        return Station { name: "".to_string() , departures };
    }
    let name = responses.first().unwrap().event.call.stop.name.text.clone();
    for response in responses {
        departures.push(trias_stop_event_to_departure(response));
    }
    Station{ name, departures }
}

fn trias_stop_event_to_departure(event: StopEventResult) -> Departure {
    let mut is_delayed = false;
    let raw_dep_time = event.event.call.stop.departure.original_time;
    let est_time = match event.event.call.stop.departure.estimated_time {
        Some(value) => {
            if value != raw_dep_time {
                is_delayed = true;
                time_format(value)
            } else {
                "".to_string()
            }
        }
        None => {
            "".to_string()
        }
    };
    Departure {
        destination: event.event.service.dest_stop_point_name.text,
        dep_time: time_format(raw_dep_time),
        real_time: est_time,
        is_delayed,
        line: event.event.service.service_section.line_name.text,
        cancelled: false
    }
}

fn time_format(time: DateTime<Utc>) -> String {
    let local_time = time.with_timezone(&Berlin);
    format!("{:02}:{:02}", local_time.hour(), local_time.minute())
}

