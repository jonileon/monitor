use reqwest::{Client};
use core::str;
use std::env::{self};
use serde::Deserialize;
use crate::train_widget::widget::{Departure, Station};

pub fn parse_departures_for_widget() -> Station {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let response = rt.block_on(async {get_departures_from_api().await});
    jsondepartureresponse_to_station(response.unwrap())
}

// Structs for the Token Api Response
#[derive(Debug, Deserialize)]
struct JsonTokenResponse {
    pub success: Vec<SuccessItem>,
}

#[derive(Debug, Deserialize)]
struct SuccessItem {
    token: String,
}


// Structs for the Departure Api Response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonDepartureResponse {
    stop_name: String,
    departures: Vec<JsonDeparture>,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonDeparture {
    route: String,
    destination: String,
//    product: String,
    time: String,
    #[serde(rename = "realtime_time")]
    realtime_time: String,
    delay_known: bool,
    cancelled: bool,
}


async fn get_departures_from_api() -> Result<JsonDepartureResponse, reqwest::Error> {
    let apikey =env::var("TRAIN_APIKEY").unwrap();
    let tokenurl =env::var("TRAIN_TOKEN_URL").unwrap();
    let referer = env::var("TRAIN_REFERER").unwrap();
    let url = env::var("TRAIN_DEPARTURES_URL").unwrap();
    let client = Client::new();
    let token_response = client.get(tokenurl).header(reqwest::header::CONTENT_TYPE, "application/json").header("Apikey", &apikey).send().await?.json::<JsonTokenResponse>().await;
    let token: &str = &token_response.unwrap().success.first().unwrap().token.clone();
    let response = client.get(url).header("Signature", token).header("Referer", referer).header(reqwest::header::CONTENT_TYPE, "application/json").header("Apikey", &apikey).send().await?.json::<JsonDepartureResponse>().await;
    return response;
}

fn jsondepartureresponse_to_station(response: JsonDepartureResponse) -> Station{
    let departures = response.departures.into_iter().map(|obj| jsondeparture_to_departure(obj)).collect();
    return Station{name: response.stop_name, departures: departures};
}

fn jsondeparture_to_departure (response: JsonDeparture)-> Departure{
    return Departure{
        destination: response.destination,
        dep_time: convert_time_format(response.time),
        is_delayed: response.delay_known,
        real_time: convert_time_format(response.realtime_time),
        line: response.route,
        cancelled: response.cancelled,
    };
}

fn convert_time_format (time: String) -> String {
    let mut time_h_m_s = time.split_whitespace().nth(1).unwrap().split(':');
    return time_h_m_s.nth(0).unwrap().to_string() + ":" + time_h_m_s.nth(0).unwrap();
}

