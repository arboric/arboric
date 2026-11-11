use rocket::{Config, launch};

#[launch]
fn rocket() -> _ {
    let figment = Config::figment()
        .merge(("address", "0.0.0.0"))
        .join(("port", 3001));
    heroes_api::build_rocket().configure(figment)
}
