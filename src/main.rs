use tabled::settings::{style::BorderColor, Color, Style};

fn main() {
    let matrix = csv::parse("AAA,AA,A
BBB,BB,B");

    dbg!(&matrix);
    let mut table = tabled::builder::Builder::from_iter(matrix).build();

    table.with(Style::rounded());
    // .with(BorderColor::filled(Color::FG_GREEN))

    println!("{table}");
}
