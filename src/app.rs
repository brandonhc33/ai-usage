// SPDX-License-Identifier: MIT

use crate::format;
use crate::icons::IconKind;
use crate::status::{self, ClaudeStatus, CodexStatus, StatusState, UsageMetric};
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Limits, Rectangle, Subscription, time};
use cosmic::prelude::*;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::{theme, widget};
use std::process::Command;
use std::time::Duration;

const CODEX_USAGE_URL: &str = "https://chatgpt.com/codex/cloud/settings/analytics#usage";
const CLAUDE_USAGE_URL: &str = "https://claude.ai/new#settings/usage";

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Id of the currently open popup, if any.
    popup: Option<Id>,
    /// Last status read from `~/.cache/ai-usage-status/status.json`.
    status: StatusState,
    /// Whether `ai-usage-collect` is currently running.
    loading: bool,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    PopupClosed(Id),
    Tick,
    Surface(cosmic::surface::Action),
    RunCollector,
    CollectorFinished,
    RunLogin(&'static str),
    OpenLink(&'static str),
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.brandonhc33.ai-usage";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            status: status::load_status(),
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    fn view(&self) -> Element<'_, Self::Message> {
        let entries = format::topbar_entries(&self.status);
        let suggested = self.core.applet.suggested_size(true);
        let (padding_major, padding_minor) = self.core.applet.suggested_padding(true);
        let vertical_padding = if self.core.applet.is_horizontal() {
            padding_minor
        } else {
            padding_major
        };

        let mut content = widget::Row::new().align_y(Alignment::Center).spacing(10);

        for entry in &entries {
            let icon = widget::icon(entry.icon.handle())
                .width(Length::Fixed(f32::from(suggested.0)))
                .height(Length::Fixed(f32::from(suggested.1)));

            let label = widget::text(entry.label.clone()).size(14);
            let label = match entry.severity.color() {
                Some(color) => label.class(theme::Text::Color(color)),
                None => label,
            };

            content = content.push(
                widget::Row::new()
                    .align_y(Alignment::Center)
                    .spacing(4)
                    .push(icon)
                    .push(label),
            );
        }

        if self.loading {
            content = content.push(
                widget::indeterminate_circular()
                    .size(f32::from(suggested.1))
                    .bar_height(2.0),
            );
        }

        let tooltip = entries
            .iter()
            .map(|entry| entry.tooltip.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let button_height = f32::from(suggested.1 + 2 * vertical_padding);

        let button = widget::button::custom(widget::layer_container(content).center(Length::Fill))
            .height(Length::Fixed(button_height))
            .padding([0, padding_major])
            .class(theme::Button::AppletIcon);

        let have_popup = self.popup;
        let button = button.on_press_with_rectangle(move |offset, bounds| {
            if let Some(id) = have_popup {
                Message::Surface(destroy_popup(id))
            } else {
                Message::Surface(app_popup::<AppModel>(
                    move |state: &mut AppModel| {
                        let new_id = Id::unique();
                        state.popup = Some(new_id);
                        let mut popup_settings = state.core.applet.get_popup_settings(
                            state.core.main_window_id().unwrap(),
                            new_id,
                            None,
                            None,
                            None,
                        );

                        popup_settings.positioner.anchor_rect = Rectangle {
                            x: (bounds.x - offset.x) as i32,
                            y: (bounds.y - offset.y) as i32,
                            width: bounds.width as i32,
                            height: bounds.height as i32,
                        };

                        // Wider than the default 360px so long reset labels
                        // ("reinicia Jun 18, 2026 3:29 PM") fit on one line.
                        popup_settings.positioner.size_limits = Limits::NONE
                            .min_width(640.0)
                            .max_width(640.0)
                            .min_height(1.0)
                            .max_height(1080.0);

                        popup_settings
                    },
                    Some(Box::new(|state: &AppModel| {
                        // `popup_container` itself wraps its content in an
                        // `Autosize` hardcoded to max_width(360.0), which
                        // overrides the positioner's size_limits above and
                        // wraps long labels. Override it back to 640px.
                        Element::from(
                            state
                                .core
                                .applet
                                .popup_container(popup_view(state))
                                .min_width(640.0)
                                .max_width(640.0),
                        )
                        .map(cosmic::Action::App)
                    })),
                ))
            }
        });

        // The panel gives applet surfaces an icon-sized window by default,
        // which clips a multi-provider Row of icons + percentages. Autosize
        // the window to fit the content instead.
        self.core
            .applet
            .autosize_window(self.core.applet.applet_tooltip(
                button,
                tooltip,
                self.popup.is_some(),
                Message::Surface,
                self.core.main_window_id(),
            ))
            .into()
    }

    /// The applet's popup is built from `popup_view`, not from a separate window.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        widget::text("").into()
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_secs(30)).map(|_| Message::Tick)
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Tick => {
                self.status = status::load_status();
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::RunCollector => {
                if self.loading {
                    return Task::none();
                }
                self.loading = true;
                return cosmic::task::future(async {
                    run_collector().await;
                    Message::CollectorFinished
                });
            }
            Message::CollectorFinished => {
                self.loading = false;
                self.status = status::load_status();
            }
            Message::RunLogin(provider) => {
                run_login(provider);
            }
            Message::OpenLink(url) => {
                open_link(url);
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

/// Builds the popup content from the current application state. Called by
/// the COSMIC runtime every time the popup needs to be (re)rendered, so it
/// always reflects the latest `status.json`.
fn popup_view(state: &AppModel) -> Element<'_, Message> {
    let mut content = widget::Column::new().spacing(12).padding(16);

    content = content.push(widget::text::heading("AI Usage"));

    match &state.status {
        StatusState::Missing => {
            content = content
                .push(widget::text::body(
                    "No se encontró ~/.cache/ai-usage-status/status.json",
                ))
                .push(widget::text::caption(
                    "Ejecuta ai-usage-collect para generar datos.",
                ));
        }
        StatusState::Invalid => {
            content = content.push(widget::text::body(
                "status.json existe pero no es válido.",
            ));
        }
        StatusState::Loaded(status) => {
            content = content.push(widget::text::caption(format::updated_at_line(status)));

            if let Some(codex) = &status.codex {
                content = content.push(codex_card(codex));
            }

            if let Some(claude) = &status.claude {
                content = content.push(claude_card(claude));
            }

            if !status.errors.is_empty() {
                let mut errors = widget::Column::new().spacing(4);
                for error in &status.errors {
                    errors = errors
                        .push(widget::text::caption(format!(
                            "{}: {}",
                            error.provider, error.message
                        )))
                        .push(widget::text::caption(format::error_hint(error)));
                }
                content = content.push(errors);
            }
        }
    }

    content = content.push(widget::divider::horizontal::default());

    let update_button = widget::button::standard(if state.loading {
        "Actualizando…"
    } else {
        "Actualizar ahora"
    })
    .width(Length::Fill);
    let update_button = if state.loading {
        update_button
    } else {
        update_button.on_press(Message::RunCollector)
    };

    let update_row: Element<'_, Message> = if state.loading {
        widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(update_button)
            .push(widget::indeterminate_circular().size(20.0).bar_height(3.0))
            .into()
    } else {
        update_button.into()
    };

    let actions = widget::Column::new()
        .spacing(8)
        .push(update_row)
        .push(
            widget::button::text("Iniciar sesión Codex")
                .on_press(Message::RunLogin("codex"))
                .width(Length::Fill),
        )
        .push(
            widget::button::text("Iniciar sesión Claude")
                .on_press(Message::RunLogin("claude"))
                .width(Length::Fill),
        );

    let links = widget::Row::new()
        .spacing(8)
        .push(
            widget::button::text("Panel Codex")
                .on_press(Message::OpenLink(CODEX_USAGE_URL))
                .width(Length::Fill),
        )
        .push(
            widget::button::text("Panel Claude")
                .on_press(Message::OpenLink(CLAUDE_USAGE_URL))
                .width(Length::Fill),
        );

    content = content.push(actions).push(links);

    content.into()
}

fn codex_card(codex: &CodexStatus) -> Element<'_, Message> {
    let mut card = widget::Column::new()
        .spacing(4)
        .push(card_header("Codex", IconKind::Codex));

    if codex.ok {
        card = card
            .push(metric_row("5 horas", &codex.five_hour))
            .push(metric_row("Semanal", &codex.weekly))
            .push(
                widget::Row::new()
                    .spacing(8)
                    .push(widget::text("Créditos").width(Length::Fixed(64.0)))
                    .push(widget::text(format::credits(codex.credits_remaining))),
            );
    } else {
        card = card.push(widget::text::caption(
            codex
                .error_message
                .clone()
                .unwrap_or_else(|| "Codex no disponible.".to_string()),
        ));
    }

    widget::container(card).padding(8).into()
}

fn claude_card(claude: &ClaudeStatus) -> Element<'_, Message> {
    let mut card = widget::Column::new()
        .spacing(4)
        .push(card_header("Claude", IconKind::Claude));

    if claude.ok {
        card = card
            .push(metric_row("Sesión", &claude.session))
            .push(metric_row("Semanal", &claude.weekly))
            .push(
                widget::Row::new()
                    .spacing(8)
                    .push(widget::text("Rutinas").width(Length::Fixed(64.0)))
                    .push(widget::text(format::routines(&claude.daily_routines))),
            );
    } else {
        card = card.push(widget::text::caption(
            claude
                .error_message
                .clone()
                .unwrap_or_else(|| "Claude no disponible.".to_string()),
        ));
    }

    widget::container(card).padding(8).into()
}

fn card_header<'a>(title: &'a str, icon: IconKind) -> Element<'a, Message> {
    widget::Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(
            widget::icon(icon.handle())
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0)),
        )
        .push(widget::text::heading(title))
        .into()
}

fn metric_row<'a>(label: &'a str, metric: &Option<UsageMetric>) -> Element<'a, Message> {
    let usage = widget::text(format::usage_summary(metric));
    let usage = match format::metric_severity(metric).color() {
        Some(color) => usage.class(theme::Text::Color(color)),
        None => usage,
    };

    let reset = widget::text::caption(format!("reinicia {}", format::reset_label(metric)))
        .class(theme::Text::Color(format::RESET_LABEL_COLOR));

    widget::Column::new()
        .push(
            widget::Row::new()
                .spacing(8)
                .push(widget::text(label).width(Length::Fixed(64.0)))
                .push(usage),
        )
        .push(reset)
        .into()
}

/// Runs the external collector and waits for it to finish. The applet never
/// scrapes or logs in itself; see docs/docs/02-architecture.md.
async fn run_collector() {
    if let Err(why) = tokio::process::Command::new("ai-usage-collect")
        .status()
        .await
    {
        eprintln!("ai-usage-applet: no se pudo ejecutar ai-usage-collect: {why}");
    }
}

/// Launches the external login helper, which opens a visible Chromium window
/// for the user to sign in and persists the session for the collector.
fn run_login(provider: &str) {
    if let Err(why) = Command::new("ai-usage-auth").arg(provider).spawn() {
        eprintln!("ai-usage-applet: no se pudo ejecutar ai-usage-auth {provider}: {why}");
    }
}

/// Opens a usage dashboard URL in the user's default browser.
fn open_link(url: &str) {
    if let Err(why) = Command::new("xdg-open").arg(url).spawn() {
        eprintln!("ai-usage-applet: no se pudo abrir el navegador: {why}");
    }
}
