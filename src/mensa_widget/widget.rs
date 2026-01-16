use ratatui::{
    prelude::{Line, Span,  Style, Text}, style::{Color, Modifier}, widgets::Widget
};

const NO_MEALS_TODAY: &str = "Die Mensa hat heute nicht offen :(";

pub struct MensaWidget {
    pub lines: Vec<MensaLine>
}
impl Widget for MensaWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut text = Text::from(" ");
        if self.lines.len() == 0 {
            text.push_line(Line::styled(NO_MEALS_TODAY, Style::default().fg(Color::Yellow)).centered());
        }
        for line in self.lines {
            let title = Line::styled(line.name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
            text.push_line(title);
            let price_padding_length = 6;
            for meal in line.meals {
                text = render_meal(text, meal, price_padding_length, usize::from(area.width));
            }
            text.extend(Line::from(" "));
        }
        text.render(area, buf);
    }
}

const PP100G: f64 = 1.1;
fn render_meal (mut text: Text, meal: MensaMeal, price_padding_length: usize, width: usize) -> Text {
    let mut price_padding = " ".repeat(price_padding_length);
    if meal.price == PP100G {
        price_padding = " 100g ".to_string();
    }
    let price_string =  price_padding + (format!("{:.2}", meal.price)).as_str();
    let len = meal.name.chars().count() + price_string.len();
    let mut amount = 0;
    let mut name = Span::raw(meal.name.clone());
    if (width as i16) - (len as i16) > 0 {
        amount =  usize::from(width) - len;
    } else {
        let name_length = (usize::from(width)) - (amount + price_string.len() + 3);
        let name_trimmed: String = meal.name.chars().into_iter().take(name_length).collect();
        let name_string = (name_trimmed + "...").to_string();
        name = Span::raw(name_string);
    }
    let padding = Span::raw(" ".repeat(amount));
    let price = Span::styled(
        price_string,
        Style::default().add_modifier(Modifier::BOLD)
    );
    text.push_line(Line::from_iter([
        name,
        padding,
        price
    ].into_iter()));
    return text;
}

#[derive(Clone)]
pub struct Mensa {
    pub name: String,
    pub lines: Vec<MensaLine>,
    pub abendausgabe_open: bool,
}

#[derive(Clone)]
pub struct MensaLine {
    pub name: String,
    pub meals: Vec<MensaMeal>,
}

#[derive(Clone)]
pub struct MensaMeal {
    pub name: String,
    pub price: f64,
//    pub notes: Vec<String>
}
