use sabai::Result;
use sabai::log::Logging;

fn main() -> Result<()> {
    let config = blog::config()?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    let app = blog::app(config)?;
    match std::env::args().nth(1).as_deref() {
        Some("migrate") => sabai::orm::migrate(&app, &blog::migrations()),
        _ => sabai::http::serve(app, blog::routes()),
    }
}
