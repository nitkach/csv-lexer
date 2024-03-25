use tabled::settings::{style::BorderColor, Color, Style};
use serde_json::value;

fn main() {
    let matrix = csv::parse(r#"1,"foo"
3,"foo""bar""""#);

    dbg!(&matrix);
    let mut table = tabled::builder::Builder::from_iter(matrix).build();

    table.with(Style::rounded());
    // .with(BorderColor::filled(Color::FG_GREEN))

    println!("{table}");
}
