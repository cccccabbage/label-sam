use app::App;

mod app;
mod config;
mod utils;

fn main() {
    let config = config::Config::new();
    App::new(config).run();
}
