// SPDX-License-Identifier: MPL-2.0

use crate::checker;
use crate::config;
use crate::config::AppConfig;

use cosmic::app::Task;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle, Subscription};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use cosmic::Element;

const APP_ID: &str = "dev.cosmic.ProxyStatus";

// Icon names — fall back to a standard symbolic icon available in all themes
const ICON_CONNECTED: &str = "network-transmit-receive-symbolic";
const ICON_CHECKING: &str = "network-transmit-symbolic";
const ICON_DISCONNECTED: &str = "network-offline-symbolic";

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    CheckDone(Result<checker::CheckResult, String>),
    TogglePoll,
    SaveSettings,
    UrlInput(String),
    CheckUrlInput(String),
    IntervalInput(String),
    ConfigUpdated(AppConfig),
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
}

pub struct StatusInfo {
    pub ok: bool,
    pub latency_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub checked_at: String,
}

pub struct ProxyStatusApp {
    core: cosmic::Core,
    config: AppConfig,
    status: Option<StatusInfo>,
    checking: bool,
    poll_active: bool,
    popup: Option<Id>,
    url_input: String,
    check_input: String,
    interval_input: String,
}

impl ProxyStatusApp {
    fn run_check(&mut self) -> Task<Message> {
        self.checking = true;
        let proxy_url = self.config.proxy_url.clone();
        let check_url = self.config.check_url.clone();
        cosmic::task::future(async move {
            let result = if !proxy_url.is_empty() {
                checker::check_via_proxy(&proxy_url, &check_url).await
            } else {
                checker::check_direct(&check_url).await
            };
            Message::CheckDone(result)
        })
    }

    fn update_status(&mut self, result: Result<checker::CheckResult, String>) {
        self.checking = false;
        match result {
            Ok(r) => {
                self.status = Some(StatusInfo {
                    ok: r.ok,
                    latency_ms: r.latency_ms,
                    status_code: r.status_code,
                    error: r.error,
                    checked_at: r.checked_at,
                });
            }
            Err(e) => {
                self.status = Some(StatusInfo {
                    ok: false,
                    latency_ms: None,
                    status_code: None,
                    error: Some(e),
                    checked_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }
    }

    fn current_icon(&self) -> &'static str {
        if self.checking {
            ICON_CHECKING
        } else if self.status.as_ref().is_some_and(|s| s.ok) {
            ICON_CONNECTED
        } else {
            ICON_DISCONNECTED
        }
    }

    fn popup_content(&self) -> Element<'_, Message> {
        let cosmic::app::Core { applet, .. } = &self.core;

        // ── Status section ────────────────────────────────────────────────
        let status_label = if self.checking {
            "Checking..."
        } else if self.status.as_ref().is_some_and(|s| s.ok) {
            "Connected"
        } else if self.status.is_some() {
            "Disconnected"
        } else {
            "Not checked"
        };

        let mut col = widget::column::with_capacity(10).spacing(4);

        col = col.push(
            widget::row::with_children(vec![
                widget::icon::from_name(self.current_icon())
                    .size(16)
                    .into(),
                widget::text::body(status_label).into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center),
        );

        if let Some(info) = &self.status {
            if let Some(ms) = info.latency_ms {
                col = col.push(widget::text::caption(format!("Latency: {ms} ms")));
            }
            if let Some(code) = info.status_code {
                col = col.push(widget::text::caption(format!("HTTP {code}")));
            }
            if let Some(ref err) = info.error {
                if !err.is_empty() {
                    col = col.push(widget::text::caption(err.as_str()));
                }
            }
            col = col.push(widget::text::caption(format!("Checked: {}", info.checked_at)));
        }

        col = col.push(widget::divider::horizontal::default());

        // ── Toggle poll button ────────────────────────────────────────────
        col = col.push(
            widget::button::text(if self.poll_active { "Stop Monitoring" } else { "Start Monitoring" })
                .on_press(Message::TogglePoll)
                .width(Length::Fill),
        );

        col = col.push(widget::divider::horizontal::default());

        // ── Settings section ──────────────────────────────────────────────
        col = col.push(widget::text::body("Settings"));

        col = col.push(widget::text::caption("Proxy URL"));
        col = col.push(
            widget::text_input("http://host:port", &self.url_input)
                .on_input(Message::UrlInput)
                .width(Length::Fill),
        );

        col = col.push(widget::text::caption("Check URL"));
        col = col.push(
            widget::text_input("http://example.com", &self.check_input)
                .on_input(Message::CheckUrlInput)
                .width(Length::Fill),
        );

        col = col.push(widget::text::caption("Interval (seconds, 1–300)"));
        col = col.push(
            widget::text_input("10", &self.interval_input)
                .on_input(Message::IntervalInput)
                .width(Length::Fill),
        );

        col = col.push(
            widget::button::text("Save")
                .on_press(Message::SaveSettings)
                .width(Length::Fill),
        );

        Element::from(applet.popup_container(col))
    }
}

impl cosmic::Application for ProxyStatusApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = cosmic::cosmic_config::Config::new(APP_ID, AppConfig::VERSION)
            .map(|ctx| match AppConfig::get_entry(&ctx) {
                Ok(c) => c,
                Err((_, c)) => c,
            })
            .unwrap_or_default();

        let app = Self {
            url_input: config.proxy_url.clone(),
            check_input: config.check_url.clone(),
            interval_input: config.interval_secs.to_string(),
            core,
            config,
            status: None,
            checking: false,
            poll_active: false,
            popup: None,
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Tick => {
                if self.poll_active && !self.checking {
                    return self.run_check();
                }
            }
            Message::CheckDone(result) => {
                self.update_status(result);
            }
            Message::TogglePoll => {
                self.poll_active = !self.poll_active;
                if self.poll_active && !self.checking {
                    return self.run_check();
                }
            }
            Message::SaveSettings => {
                let interval = self
                    .interval_input
                    .parse::<u64>()
                    .unwrap_or(self.config.interval_secs)
                    .clamp(1, 300);
                self.config.proxy_url = self.url_input.clone();
                self.config.check_url = self.check_input.clone();
                self.config.interval_secs = interval;
                if let Err(e) = config::save_config(APP_ID, &self.config) {
                    eprintln!("Failed to save config: {e}");
                }
            }
            Message::UrlInput(s) => self.url_input = s,
            Message::CheckUrlInput(s) => self.check_input = s,
            Message::IntervalInput(s) => self.interval_input = s,
            Message::ConfigUpdated(new_config) => {
                self.config = new_config.clone();
                self.url_input = new_config.proxy_url.clone();
                self.check_input = new_config.check_url.clone();
                self.interval_input = new_config.interval_secs.to_string();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let popup_open = self.popup.is_some();
        let icon_name = self.current_icon();

        let btn = self
            .core
            .applet
            .icon_button(icon_name)
            .on_press_with_rectangle(move |offset, bounds| {
                if popup_open {
                    Message::Surface(destroy_popup(
                        // will be cleaned up by PopupClosed
                        cosmic::iced::window::Id::NONE,
                    ))
                } else {
                    Message::Surface(app_popup::<ProxyStatusApp>(
                        move |state: &mut ProxyStatusApp| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );
                            settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            settings
                        },
                        Some(Box::new(|state: &ProxyStatusApp| {
                            state.popup_content().map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        Element::from(self.core.applet.applet_tooltip(
            btn,
            "Proxy Status",
            popup_open,
            Message::Surface,
            None,
        ))
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        widget::text("").into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let config_sub = self
            .core()
            .watch_config::<AppConfig>(APP_ID)
            .map(|update| Message::ConfigUpdated(update.config));

        if self.poll_active {
            let interval = std::time::Duration::from_secs(self.config.interval_secs);
            let timer_sub = cosmic::iced::time::every(interval).map(|_| Message::Tick);
            Subscription::batch(vec![config_sub, timer_sub])
        } else {
            config_sub
        }
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
