//! Root view of the separate settings window. All state lives in
//! `BotView`; this host only renders its settings UI into a second window.

use gpui_kit::{Context, Entity, IntoElement, Render, Window};

use super::view::BotView;

pub struct SettingsHost {
    pub bot: Entity<BotView>,
}

impl Render for SettingsHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bot = self.bot.clone();
        bot.update(cx, |bot, bot_cx| bot.render_settings_window(window, bot_cx))
    }
}
