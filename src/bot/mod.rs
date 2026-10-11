mod live_map;
mod module_settings;
mod settings_window;
#[cfg(windows)]
mod tray;
mod view;

use std::sync::Arc;

use gpui_kit::component::{Root, Theme, ThemeMode};
use gpui_kit::{
    App, AppContext, Bounds, TitlebarOptions, WindowBounds, WindowDecorations, WindowOptions, px,
    size,
};
use gpui_kit::{application, init};

use view::BotView;
use xenomit_bot_runtime::behavior::PlannerRuntime;

pub fn run() {
    application().run(|cx: &mut App| {
        init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        let bounds = Bounds::centered(None, size(px(620.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // Borderless on Windows/macOS: the app draws its own titlebar
                // (see BotView::render_titlebar); Windows keeps native resize,
                // snap and caption-button behavior via window control areas.
                titlebar: Some(TitlebarOptions {
                    title: Some("Xenomit — Sign in".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(600.0), px(520.0))),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("Xenomit — Sign in");
                let view = cx
                    .new(|cx| BotView::new(window, cx, Some(Arc::new(PlannerRuntime::default()))));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("could not open Xenomit Bot window");
        cx.activate(true);
    });
}
