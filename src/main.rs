mod app;
mod checker;
mod config;

fn main() -> cosmic::iced::Result {
    env_logger::init();
    cosmic::applet::run::<app::ProxyStatusApp>(())
}
