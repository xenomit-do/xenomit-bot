//! Settings persistence helpers.

use super::*;

pub(super) fn settings_path() -> PathBuf {
    FileSettingsStore::default_path()
}

impl BotView {
    pub(super) fn save_settings(&self) {
        let _ = self.settings_store.save(&self.settings);
        // A running module holds a config snapshot from Start; push the
        // updated one so NPC list toggles and feature flags apply live.
        if self.behavior_running
            && let Some(engine) = &self.engine
        {
            engine.update(&self.settings.module_config());
        }
        self.push_panel_config();
    }
}
