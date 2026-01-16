use reqwest::Client;
use std::env;
use serde::Deserialize;

pub fn get_weather_today() -> Weather {
    let rt = tokio::runtime::Runtime::new().unwrap();
    return rt
        .block_on(async {get_weather_from_api().await})
        .expect("Error from Weather Api");
}

async fn get_weather_from_api() -> Result<Weather, reqwest::Error> {
    let url = env::var("WEATHER_URL").expect("No Url for the Weather found");
    let client = Client::new();
    let response = client.get(url).send().await?.json::<Weather>().await;
    return response;
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct Weather{
    pub hourly: HourlyResponse,
    pub daily: DailyResponse,
    pub current: CurrentResponse,
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct HourlyResponse {
    pub time: Vec<String>,
    pub weather_code: Vec<u8>,
    pub temperature_2m: Vec<f32>,
    pub precipitation: Vec<f32>,
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct DailyResponse {
    pub time: Vec<String>,
    pub precipitation_sum: Vec<f32>,
    pub weather_code: Vec<u8>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
}

#[derive(Clone)]
#[derive(Deserialize)]
pub struct CurrentResponse {
    pub weather_code: u8,
    pub temperature_2m: f32,
}

