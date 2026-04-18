use chrono::Local;
use reqwest::Client;
use std::env;
use serde::Deserialize;
use crate::mensa_widget::widget::{Mensa, MensaLine, MensaMeal};

pub fn get_meals_today() -> Mensa {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut abendausgabe_open = false;
    let response = rt
        .block_on(async {get_meals_from_api().await})
        .expect("Error from Mensa Api");
    let lines = parse_mensa_meals_to_line(response, &mut abendausgabe_open);
    return Mensa{name: "Mensa am Adenauer Ring".to_string(), lines: lines, abendausgabe_open: abendausgabe_open};
}

async fn get_meals_from_api() -> Result<Vec<JsonMensaMeal>, reqwest::Error> {
    let urls_wo_date =env::var("MENSA_URL").expect("No Url for the Mensa found");
    let now = Local::now()
        .to_string();
    let current_date = now
        .split_whitespace()
        .nth(0).unwrap();
    let url = urls_wo_date + current_date + "/meals";
    let client = Client::new();
    let response = client.get(url).send().await?.json::<Vec<JsonMensaMeal>>().await;
    return response;
}

const EXCLUDE: [&str; 2] = ["Cafeteria 11-14 Uhr", "[pizza]werk Salate / Vorspeisen"];
fn parse_mensa_meals_to_line(meals: Vec<JsonMensaMeal>, abendausgabe_open: &mut bool) -> Vec<MensaLine> {
    let mut lines: Vec<MensaLine> = Vec::new();
    for meal in meals {
        if !EXCLUDE.contains(&meal.category.as_str()) {
            if meal.category.contains("Abendessen") && meal.name.contains("Abendessen"){
                *abendausgabe_open = true;
            } else {
                let mut line_exists = false;
                for line in &mut lines {
                    if meal.category == line.name {
                        line.meals.push(MensaMeal{
                            name: meal.name.clone(),
                            price: meal.prices.students
                        });
                        line_exists = true;
                    }
                }
                if !line_exists {
                    let mut meals: Vec<MensaMeal> = Vec::new();
                    meals.push(MensaMeal{
                        name: meal.name.clone(),
                        price: meal.prices.students
                    });
                    lines.push(MensaLine{name: meal.category, meals: meals})
                }
            }
        }
    }
    return lines;
}

#[derive(Deserialize)]
pub struct JsonMensaMeal {
    pub name: String,
    pub category: String,
    pub prices: JsonPrice,
}

#[derive(Deserialize)]
pub struct JsonPrice {
    pub students: f64
}

