// SPDX-License-Identifier: MIT

mod app;
mod format;
mod icons;
mod status;

fn main() -> cosmic::iced::Result {
    // Starts the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<app::AppModel>(())
}
