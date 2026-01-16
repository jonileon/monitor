use itertools::izip;

use chrono::{NaiveDateTime, TimeDelta};
use ratatui::{
    layout::Rect, prelude::{Line, Span,  Style, Text}, style::{Color, Modifier}, symbols, widgets::{Block, Axis, Borders, Chart, Dataset, Widget}
};
use crate::weather_widget::widget_backend::{DailyResponse, Weather};

const CELSIUS: &str = "°C";

pub struct WeatherWidget {
    pub weather: Weather
}
impl Widget for WeatherWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        render_current_temp(area, buf, &self.weather);
        let temp_graph_area = Rect::new(area.x + 1, area.y + 4, area.width/2, 10);
        let rain_graph_area = Rect::new(area.x + 1 + area.width/2, area.y + 4, if area.width%2 == 0 {area.width/2 - 2} else {area.width/2 - 1}, 10);
        draw_graph("Temperaturen", &self.weather.hourly.time, self.weather.hourly.temperature_2m, temp_graph_area, buf, Color::Red);
        draw_graph("Niederschlag", &self.weather.hourly.time, self.weather.hourly.precipitation, rain_graph_area, buf, Color::Blue);
        render_week_forecast(Rect::new(area.x, area.y + 14, area.width, 15), buf, self.weather.daily);
    }
}

fn draw_graph(title: &str, times: &Vec<String>, data: Vec<f32>, area: Rect, buf: &mut ratatui::prelude::Buffer, color: Color){
    let parsed_times: Vec<NaiveDateTime> = times
        .iter()
        .map(|t| NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M").unwrap())
        .collect();
    let start = parsed_times[0];
    let mut max_bound = 0.0;
    let mut min_bound = 0.0;
    let points: Vec<(f64, f64)> = parsed_times
        .iter().take(24)
        .zip(data.iter().take(24))
        .map(|(time, datum)| {
            if datum > &mut max_bound {
                max_bound = datum.clone();
            }
            if datum < &mut min_bound {
                min_bound = datum.clone();
            }
            let hours_since_start =
            (*time - start).num_minutes() as f64 / 60.0;
            (hours_since_start, *datum as f64)
        })
        .collect();
    let dataset = Dataset::default()
        .name("graph data")
        .marker(symbols::Marker::Braille)
        .graph_type(ratatui::widgets::GraphType::Line)
        .style(Style::default().fg(color))
        .data(&points);
    let first = start.time().format("%H:%M").to_string();
    let fquarter = start.checked_add_signed(TimeDelta::hours(8)).unwrap().time().format("%H:%M").to_string();
    let squarter = start.checked_add_signed(TimeDelta::hours(16)).unwrap().time().format("%H:%M").to_string();
    let end = start.checked_add_signed(TimeDelta::hours(24)).unwrap().time().format("%H:%M").to_string();
    let chart = Chart::new(vec![dataset])
        .block(Block::default().title(title).borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .bounds([0.0, points.last().unwrap().0])
                .labels(vec![
                    Span::styled(first, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(fquarter),
                    Span::raw(squarter),
                    Span::styled(end, Style::default().add_modifier(Modifier::BOLD)),
                ]),
        )
        .y_axis(Axis::default()
            .bounds([(min_bound - 1.0) as f64, (max_bound + 1.0) as f64])
            .labels(vec![
                Span::raw(format!("{:5.1}", min_bound)),
                Span::raw(format!("{:5.1}", max_bound)),
            ])
        );
    chart.render(area, buf);
}

fn render_current_temp(area: Rect, buf: &mut ratatui::prelude::Buffer, weather: &Weather){
    let mut text = Text::from(" ");
    let temp_string = format!("  {}", weather.current.temperature_2m) + CELSIUS;
    let temp_span = Span::styled(temp_string.clone(), Style::default().fg(Color::Red));
    let weather_condition = parse_wmo_code(weather.current.weather_code);
    let description = Span::raw(weather_condition.description.clone() + ": ");
    let icon = Span::styled(weather_condition.icon, Style::default().fg(Color::Blue));
    let padding = Span::from(" ".repeat(area.width as usize - 6 - weather_condition.description.len() - temp_string.chars().count()));
    text.push_line(Line::from_iter([
        temp_span,
        padding,
        description,
        icon
    ].into_iter()));
    text.push_line(Line::raw(" "));
    text.push_line(Line::from(" ".to_string() + "─".repeat(usize::from(area.width as usize - 2)).as_str()));
    text.render(area, buf);
}

fn render_week_forecast(area: Rect, buf: &mut ratatui::prelude::Buffer, forecast: DailyResponse){
    let mut text = Text::from(" ");
    izip!(forecast.time, forecast.temperature_2m_max, forecast.temperature_2m_min, forecast.precipitation_sum).into_iter().for_each(|(date, max, min, prec)| {
        let time_span = Span::raw("  ".to_string() + date.as_str() + ":");
        let rain_icon = Span::raw("󰖗  ");
        let rain_span = Span::styled(format!("{:5.1}mm", prec), Style::default().fg(Color::Blue));
        let temp_icon = Span::raw("      ");
        let temp_max_span = Span::styled(format!("{:5.1}", max) + CELSIUS, Style::default().fg(Color::Red));
        let dash = Span::raw(" - ");
        let temp_min_span = Span::styled(format!("{:5.1}", min) + CELSIUS, Style::default().fg(Color::Blue));
        let padding = Span::raw(" ".repeat(area.width as usize - 50));
        text.push_line(Line::from_iter([
            time_span,
            padding,
            rain_icon,
            rain_span,
            temp_icon,
            temp_max_span,
            dash,
            temp_min_span,
        ].into_iter()));
        text.push_line(" ");
    });
    text.render(area, buf);
}

struct WheatherCondition {
    icon: String,
    description: String,
}
fn parse_wmo_code(code: u8)-> WheatherCondition {
match code {
        0 => WheatherCondition{icon: " ".to_string(), description: "wolkenfrei".to_string()},
        1..=3 => WheatherCondition{icon: " ".to_string(), description: "bewölkt".to_string()},
        45..=48 => WheatherCondition{icon: " ".to_string(), description: " nebelig".to_string()},
        51..=57 => WheatherCondition{icon: " ".to_string(), description: "nieselig".to_string()},
        61..=67 => WheatherCondition{icon: " ".to_string(), description: "leicht regnerisch".to_string()},
        71..77 => WheatherCondition{icon: " ".to_string(), description: "leichter schnee".to_string()},
        80..=83 => WheatherCondition{icon: " ".to_string(), description: "regnerisch".to_string()},
        84..=86 => WheatherCondition{icon: " ".to_string(), description: "schnee".to_string()},
        95..=99 => WheatherCondition{icon: " ".to_string(), description: "stürmisch".to_string()},
        _ => WheatherCondition{icon:" ".to_string(), description: "unbekannt".to_string()}
    }
}
