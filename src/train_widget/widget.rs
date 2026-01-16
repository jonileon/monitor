use ratatui::{
    prelude::{Color, Line, Span,  Style, Stylize, Text},
    widgets::Widget
};
use crate::train_widget::widget_backend::parse_departures_for_widget;


pub fn get_departures() -> Station{
    parse_departures_for_widget()
}

pub fn update_departures(departures: &mut Vec<Departure>){
    let intermediate_station = get_departures();
    *departures = intermediate_station.departures
}

#[derive(Clone)]
pub struct Station {
    pub departures: Vec<Departure>,
    pub name : String,
}

#[derive(Clone)]
pub struct Departure {
    pub destination: String,
    pub dep_time: String,
    pub real_time: String,
    pub is_delayed: bool,
    pub line: String,
    pub cancelled: bool,
}

pub struct TrainWidget {
    pub departures: Vec<Departure>
}
impl Widget for TrainWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut rem_height = area.height;
        let length = area.width;
        let space_span = Span::raw(" ");
        let mut text = Text::from("");
        for departure in self.departures {
            if !departure.cancelled {
                if rem_height < 3 {
                    break;
                }

                if departure.is_delayed {
                    let fstspln = departure.dep_time.len() + 1 + departure.real_time.len() + 4 + departure.destination.chars().count();
                    if u16::try_from(fstspln).unwrap() >= length {
                        break;
                    }
                    let padding = Span::raw(" ".repeat(usize::from(length) - fstspln));
                    let dep_time_span = Span::styled(
                        departure.dep_time,
                        Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::CROSSED_OUT)
                    );
                    let to_span = Span::raw("bis ");
                    let destination_span = Span::raw(departure.destination);
                    let real_time_span = Span::raw(departure.real_time);
                    let line = Line::from_iter([
                        real_time_span,
                        space_span.clone(),
                        dep_time_span,
                        padding,
                        to_span,
                        destination_span,
                    ].into_iter());
                    text.push_line(line);
                }

                else {
                    let fstspln = departure.dep_time.len() + 4 + departure.destination.chars().count();
                    if u16::try_from(fstspln).unwrap() >= length {
                        break;
                    }
                    let to_span = Span::raw("bis ");
                    let destination_span = Span::raw(departure.destination);
                    let dep_time_span = Span::styled(departure.dep_time, Style::default().fg(Color::Green));
                    let padding = Span::raw(" ".repeat(usize::from(length) - fstspln));
                    let line = Line::from_iter([
                        dep_time_span,
                        padding,
                        to_span,
                        destination_span,
                    ].into_iter());
                    text.push_line(line);
                }

                let train_line = "═".repeat(usize::from(usize::from(length - 7)));
                let _span = train_line.magenta();
                // text.extend(Line::from(span));
                let last_line = Line::from([String::from("Bus "), departure.line.clone(),String::from(" "),"═".repeat(usize::from(usize::from(length - 6)))].concat().magenta());
                text.extend(last_line);
                text.extend(Line::from(" "));
                rem_height = rem_height - 3;
            }
        }
        text.render(area, buf);
    }
}
