mod train_widget;
mod mensa_widget;
pub mod weather_widget;
use dotenv::dotenv;
use chrono::{Local, NaiveTime, Timelike};
use color_eyre::Result;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex,
    atomic::{AtomicBool, Ordering}};
use ratatui::{
    crossterm::{event::{self, Event, KeyCode}},
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, Borders},
    DefaultTerminal, Frame
};
use crate::mensa_widget::widget::{Mensa, MensaWidget};
use crate::mensa_widget::widget_backend::get_meals_today;
use crate::train_widget::widget::{get_departures, TrainWidget, Station, update_departures};
use crate::weather_widget::widget_backend::{get_weather_today, Weather};
use crate::weather_widget::widget::WeatherWidget;

fn main() {
    let _ = dotenv();
    let terminal = ratatui::init();
    let _ = run(terminal);
    ratatui::restore();
}

fn handle_events() -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn run(mut terminal: DefaultTerminal) -> Result<()>{
    let should_quit = Arc::new(AtomicBool::new(false));
    let station_arc = Arc::new(Mutex::new(get_departures()));
    let mensa_arc = Arc::new(Mutex::new(get_meals_today()));
    let weather_arc = Arc::new(Mutex::new(get_weather_today()));

    { // weather update thread
        let weather_arc = Arc::clone(&weather_arc);        
        let quit = Arc::clone(&should_quit);
        let _handle = thread::spawn(move || {
            while !quit.load(Ordering::Relaxed) {
                {
                    let mut weather = weather_arc.lock().unwrap();
                    *weather = get_weather_today();
                }
                thread::sleep(Duration::from_secs(600));
            }
        });
    }

    { // train update thread
        let station_arc = Arc::clone(&station_arc);        
        let quit = Arc::clone(&should_quit);
        let _handle = thread::spawn(move || {
            while !quit.load(Ordering::Relaxed) {
                {
                    let mut station = station_arc.lock().unwrap();
                    update_departures(&mut station.departures);
                }
                thread::sleep(Duration::from_secs(120));
            }
        });
    }

    { // meal plan mensa update thread
        let mensa_arc = Arc::clone(&mensa_arc);        
        let quit = Arc::clone(&should_quit);
        let update_time = NaiveTime::from_hms_opt(11, 0, 0).unwrap();
        let mut update_new_day = {
            if Local::now().time().hour() <= NaiveTime::from_hms_opt(11, 0, 0).unwrap().hour() {
                false
            } else {
                true
            }
        };
        let _handle = thread::spawn(move || {
            while !quit.load(Ordering::Relaxed) {
                {
                    if (Local::now().time() > update_time && !update_new_day) || (Local::now().time() < update_time && update_new_day) {
                        let mut mensa = mensa_arc.lock().unwrap();
                        *mensa = get_meals_today();
                        update_new_day = !update_new_day;
                    }
                }
                thread::sleep(Duration::from_secs(120));
            }
        });
    }

    while !should_quit.load(Ordering::Relaxed) {
        let station_snap = {
            let station = station_arc.lock().unwrap();
            station.clone()
        };
        let mensa_snap = {
            let mensa = mensa_arc.lock().unwrap();
            mensa.clone()
        };
        let weather_snap = {
            let weather = weather_arc.lock().unwrap();
            weather.clone()
        };
        let _ = terminal.draw(|f| render_monitor(f, station_snap, mensa_snap, weather_snap));
        should_quit.store(handle_events()?, Ordering::Relaxed);
    }
    Ok(())
}

fn render_monitor(frame: &mut Frame, station: Station, mensa: Mensa, weather: Weather) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]);
    let horizontal = Layout::horizontal([Constraint::Ratio(1, 2); 2]);
    let right_vertical = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]
    );
    let [title_bar, main_area, status_bar] = vertical.areas(frame.area());
    let [left, right] = horizontal.areas(main_area);
    let [ru, rd] = right_vertical.areas(right);
    frame.render_widget(
        Block::new().borders(Borders::TOP),
        title_bar,
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP)
            .title(
                if mensa.abendausgabe_open {Span::styled("Abendausgabe ist offen", Style::default().fg(Color::Green))} 
                else {Span::styled("Abendausgabe ist zu", Color::Red)}
            )
            .title_alignment(Alignment::Center),
        status_bar,
    );
    let train_block = Block::bordered().title(station.name.clone()).title_alignment(Alignment::Center);
    let train_inner = train_block.inner(ru);
    let departures = station.departures.iter().cloned().collect();
    let mensa_block = Block::bordered().title(mensa.name).title_alignment(ratatui::layout::Alignment::Center);
    let mensa_inner = mensa_block.inner(left);
    let weather_block = Block::bordered().title("Wetter").title_alignment(ratatui::layout::Alignment::Center);
    let weather_inner = weather_block.inner(rd);

    frame.render_widget(mensa_block, left);
    frame.render_widget(MensaWidget{lines: mensa.lines}, mensa_inner);
    frame.render_widget(train_block, ru);
    frame.render_widget(TrainWidget{departures}, train_inner);
    frame.render_widget(weather_block, rd);
    frame.render_widget(WeatherWidget{weather: weather}, weather_inner);
}

