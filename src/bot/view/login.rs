//! Sign-in dialog and the authentication/connection flow.

use super::*;

impl BotView {
    pub(super) fn render_login_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .w(px(480.0))
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .bg(rgb(PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(LINE_SOFT))
            .shadow_lg()
            .on_mouse_down(
                gpui_kit::MouseButton::Left,
                cx.listener(|_, _, _, cx| cx.stop_propagation()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .size(px(24.0))
                            .rounded_sm()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(rgb(BLUE))
                            .text_size(px(13.0))
                            .text_color(rgb(0xffffff))
                            .child("◆"),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT))
                            .child("XENOMIT"),
                    ),
            )
            .child(
                div()
                    .mt(px(6.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(24.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT))
                            .child("Welcome back"),
                    )
                    .child(label("Sign in to connect your game client.", MUTED, 12.0)),
            )
            .child(section("GAME CLIENT"));
        let mut clients = div()
            .flex()
            .gap(px(4.0))
            .p(px(3.0))
            .rounded_md()
            .bg(rgb(PANEL_DARK))
            .border_1()
            .border_color(rgb(LINE_SOFT));
        for kind in [ClientKind::Ruffle, ClientKind::Flash, ClientKind::Packet] {
            let selected = self.settings.client == kind;
            clients = clients.child(
                div()
                    .id(kind.label())
                    .flex_1()
                    .py(px(8.0))
                    .rounded_md()
                    .bg(rgb(if selected { BLUE_TINT } else { PANEL_DARK }))
                    .hover(move |s| {
                        if selected {
                            s.bg(rgb(BLUE_TINT))
                        } else {
                            s.bg(rgb(PANEL_HOVER))
                        }
                    })
                    .text_center()
                    .text_size(px(13.0))
                    .font_weight(if selected {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(if selected { TEXT } else { MUTED }))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if this.busy {
                            return;
                        }
                        this.settings.client = kind;
                        this.save_settings();
                        cx.notify();
                    }))
                    .child(kind.label()),
            );
        }
        panel = panel
            .child(clients)
            .child(if self.settings.client == ClientKind::Flash {
                button(
                    "flash-capture",
                    self.settings.flash_backend.label(),
                    Btn::Ghost,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    if !this.busy && !this.session_connected {
                        this.settings.flash_backend = this.settings.flash_backend.toggle();
                        this.save_settings();
                        cx.notify();
                    }
                }))
                .into_any_element()
            } else {
                div().into_any_element()
            });
        panel = panel.child(section("SIGN IN WITH"));
        let mut modes = div()
            .flex()
            .gap(px(4.0))
            .p(px(3.0))
            .rounded_md()
            .bg(rgb(PANEL_DARK))
            .border_1()
            .border_color(rgb(LINE_SOFT));
        for (tab, title) in [
            (LoginTab::Account, "Account"),
            (LoginTab::ExistingSession, "Existing session"),
            (LoginTab::SavedCreds, "Saved creds"),
        ] {
            let selected = self.login_tab == tab;
            modes = modes.child(
                div()
                    .id(format!("login-tab-{}", title.to_ascii_lowercase().replace(' ', "-")))
                    .flex_1()
                    .py(px(7.0))
                    .rounded_sm()
                    .bg(rgb(if selected { BLUE_TINT } else { PANEL_DARK }))
                    .hover(move |s| {
                        if selected {
                            s.bg(rgb(BLUE_TINT))
                        } else {
                            s.bg(rgb(PANEL_HOVER))
                        }
                    })
                    .text_center()
                    .text_size(px(13.0))
                    .font_weight(if selected {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(if selected { TEXT } else { MUTED }))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.busy {
                            return;
                        }
                        this.login_tab = tab;
                        this.login_error = None;
                        if tab != LoginTab::SavedCreds {
                            let input = if tab == LoginTab::ExistingSession {
                                &this.server
                            } else {
                                &this.username
                            };
                            window.focus(&input.read(cx).focus_handle(cx), cx);
                        }
                        cx.notify();
                    }))
                    .child(title),
            );
        }
        panel = panel.child(modes);
        match self.login_tab {
            LoginTab::Account | LoginTab::ExistingSession => {
                let fields = if self.login_tab == LoginTab::ExistingSession {
                    vec![
                        ("Server", "server-input", &self.server),
                        ("Session dosid", "sid-input", &self.sid),
                    ]
                } else {
                    vec![
                        ("Username", "username-input", &self.username),
                        ("Password", "password-input", &self.password),
                    ]
                };
                for (index, (title, id, state)) in fields.into_iter().enumerate() {
                    panel = panel.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(5.0))
                            .child(label(title, MUTED, 12.0))
                            .child(
                                Input::new(state)
                                    .id(id)
                                    .w_full()
                                    .tab_index(index as isize + 1)
                                    .disabled(self.busy)
                                    .h(px(40.0))
                                    .text_size(px(14.0)),
                            ),
                    );
                }
                if self.login_tab == LoginTab::Account {
                    panel = panel
                        .child(label(
                            "Credentials stay in memory unless you save this account.",
                            FAINT,
                            11.5,
                        ))
                        .child(label(
                            "For accounts with CAPTCHA, choose Existing session.",
                            FAINT,
                            11.5,
                        ))
                        .child(
                            button("save-login-credentials", "Save account", Btn::Ghost)
                                .w_full()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.save_login_credentials(window, cx)
                                })),
                        );
                }
            }
            LoginTab::SavedCreds => {
                panel = panel.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.0))
                        .child(section("SAVED ACCOUNTS"))
                        .child(label("Encrypted for this Windows account", FAINT, 11.5)),
                );
                if self.saved_credentials.is_empty() {
                    panel = panel.child(
                        div()
                            .w_full()
                            .p(px(14.0))
                            .rounded_md()
                            .bg(rgb(PANEL_DARK))
                            .border_1()
                            .border_color(rgb(LINE_SOFT))
                            .child(label(
                                "No saved accounts yet. Add one from the Account tab.",
                                MUTED,
                                12.0,
                            )),
                    );
                } else {
                    let credentials = self.saved_credentials.clone();
                    let count = credentials.len();
                    let mut saved_rows = div().w_full().flex().flex_col().gap(px(7.0));
                    for (index, credential) in credentials.into_iter().enumerate() {
                        saved_rows = saved_rows.child(
                            div()
                                .w_full()
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(8.0))
                                .p(px(9.0))
                                .rounded_md()
                                .bg(rgb(PANEL_DARK))
                                .border_1()
                                .border_color(rgb(LINE_SOFT))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.0))
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .child(label(&credential.username, TEXT, 12.5)),
                                )
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .flex()
                                        .gap(px(6.0))
                                        .child(
                                            button(
                                                format!("saved-credential-connect-{index}"),
                                                "Connect",
                                                Btn::Primary,
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.sign_in_saved_credential(index, cx);
                                            })),
                                        )
                                        .child(
                                            button(
                                                format!("saved-credential-remove-{index}"),
                                                "Remove",
                                                Btn::Subtle,
                                            )
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.remove_saved_credential(index, cx);
                                            })),
                                        ),
                                ),
                        );
                    }
                    // The scrollbar viewport is `height: 100%`. This dialog is
                    // sized by its content, so that percentage resolves to 0
                    // and every account is clipped. A short list stays in
                    // normal flow. Past three accounts the scroller gets a
                    // definite height so the rest can scroll.
                    panel = panel.child(if count > 3 {
                        saved_rows
                            .id("saved-credentials-list")
                            .h(px(190.0))
                            .overflow_y_scrollbar()
                            .into_any_element()
                    } else {
                        saved_rows.into_any_element()
                    });
                }
                if let Some(error) = &self.saved_credentials_error {
                    panel = panel.child(
                        div()
                            .w_full()
                            .p(px(10.0))
                            .rounded_md()
                            .bg(rgb(RED_TINT))
                            .border_1()
                            .border_color(rgb(0x6b4046))
                            .child(label(error, 0xf0b9b9, 12.0)),
                    );
                }
            }
        }
        if let Some(error) = &self.login_error {
            panel = panel.child(
                div()
                    .w_full()
                    .p(px(11.0))
                    .rounded_md()
                    .bg(rgb(RED_TINT))
                    .border_1()
                    .border_color(rgb(0x6b4046))
                    .child(label(error, 0xf0b9b9, 12.5)),
            );
        }
        if self.login_tab != LoginTab::SavedCreds {
            panel = panel.child(
                button(
                    "submit-login",
                    if self.busy {
                        "Connecting…"
                    } else {
                        "Sign in & connect"
                    },
                    if self.busy { Btn::Subtle } else { Btn::Primary },
                )
                .w_full()
                .py(px(10.0))
                .rounded_md()
                .text_size(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .on_click(cx.listener(|this, _, window, cx| this.submit_login(window, cx))),
            );
        }
        div()
            .id("login-page")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(if self.session_connected {
                rgb(BG).alpha(0.72)
            } else {
                rgb(BG).alpha(1.0)
            })
            .font_family("Segoe UI")
            .text_color(rgb(TEXT))
            .on_mouse_down(
                gpui_kit::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.session_connected {
                        this.login_dialog_open = false;
                        cx.notify();
                    }
                }),
            )
            .capture_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "enter" {
                        cx.stop_propagation();
                        this.submit_login(window, cx);
                    }
                }),
            )
            .child(panel)
    }

    pub(super) fn submit_login(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.login_tab {
            LoginTab::ExistingSession => self.connect(cx),
            LoginTab::Account => self.submit_credentials(window, cx),
            LoginTab::SavedCreds => {}
        }
    }

    pub(super) fn save_login_credentials(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        let username = self.username.read(cx).value().trim().to_string();
        let password = self.password.read(cx).value().to_string();
        if username.is_empty() || password.is_empty() {
            self.login_error = Some("Enter both your username and password to save this account.".into());
            cx.notify();
            return;
        }

        let mut credentials = self.saved_credentials.clone();
        if let Some(existing) = credentials
            .iter_mut()
            .find(|credential| credential.username.eq_ignore_ascii_case(&username))
        {
            existing.username = username;
            existing.password = password;
        } else {
            credentials.push(SavedCredential { username, password });
        }

        match SavedCredentialStore::save(&self.saved_credentials_path, &credentials) {
            Ok(()) => {
                self.saved_credentials = credentials;
                self.saved_credentials_error = None;
                self.login_error = None;
                self.login_tab = LoginTab::SavedCreds;
                self.password
                    .update(cx, |state, cx| state.set_value("", window, cx));
            }
            Err(error) => self.saved_credentials_error = Some(error),
        }
        cx.notify();
    }

    pub(super) fn remove_saved_credential(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.busy || index >= self.saved_credentials.len() {
            return;
        }
        let mut credentials = self.saved_credentials.clone();
        credentials.remove(index);
        match SavedCredentialStore::save(&self.saved_credentials_path, &credentials) {
            Ok(()) => {
                self.saved_credentials = credentials;
                self.saved_credentials_error = None;
            }
            Err(error) => self.saved_credentials_error = Some(error),
        }
        cx.notify();
    }

    pub(super) fn sign_in_saved_credential(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if self.session_connected {
            self.login_dialog_open = false;
            self.stop_client(cx);
            cx.notify();
            return;
        }
        let Some(credential) = self.saved_credentials.get(index).cloned() else {
            return;
        };
        self.begin_authentication(
            SessionCredentials::Password {
                username: credential.username,
                password: credential.password,
            },
            cx,
        );
    }

    pub(super) fn connect(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if self.session_connected {
            self.stop_client(cx);
            cx.notify();
            return;
        }

        let server = self.server.read(cx).value().trim().to_string();
        let sid = self.sid.read(cx).value().trim().to_string();
        if server.is_empty() || sid.is_empty() {
            self.login_error = Some("Enter a server and session dosid first.".into());
            cx.notify();
            return;
        }

        self.begin_authentication(SessionCredentials::SessionId { server, sid }, cx);
    }

    pub(super) fn submit_credentials(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if self.session_connected {
            self.login_dialog_open = false;
            self.stop_client(cx);
            cx.notify();
            return;
        }
        let username = self.username.read(cx).value().trim().to_string();
        let password = self.password.read(cx).value().to_string();
        if username.is_empty() || password.is_empty() {
            self.login_error = Some("Enter both your username and password.".into());
            cx.notify();
            return;
        }
        self.password
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.begin_authentication(SessionCredentials::Password { username, password }, cx);
    }

    pub(super) fn begin_authentication(&mut self, auth: SessionCredentials, cx: &mut Context<Self>) {
        self.login_error = None;

        self.busy = true;
        self.session_label = "Authenticating…".into();
        self.activity = match &auth {
            SessionCredentials::Password { .. } => "Signing in to Backpage…".into(),
            SessionCredentials::SessionId { .. } => "Checking the session with Backpage…".into(),
        };
        cx.notify();

        let backend = self.settings.game_api_backend;
        let kind = self.settings.client;
        let flash_backend = self.settings.flash_backend;
        self.connection_generation += 1;
        let generation = self.connection_generation;
        let provider: Arc<dyn SessionProvider> = Arc::new(BackpageSessionProvider::new(auth));
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { authenticate_and_launch(provider, kind, backend, flash_backend) })
                .await;
            let monitor = result
                .as_ref()
                .ok()
                .and_then(|launch| launch.bridge.as_ref().map(|bridge| Arc::clone(bridge)));
            let _ = this.update(cx, |view, cx| {
                view.busy = false;
                match result {
                    Ok(launch) => {
                        if let Ok(mut stats) = launch.statistics.lock() {
                            stats.panel = Some(view.panel.clone());
                        }
                        view.statistics = Some(launch.statistics.clone());
                        view.panel.set_logs(launch.client_logs.clone());
                        view.session = Some(launch.session.clone());
                        behavior::set_backpage_session(Some(launch.session.clone()));
                        view.status_refreshes = 0;
                        view.last_reload = Instant::now();
                        view.session_started_secs = statistics::now_ms() / 1000;
                        view.bridge_ping_ms = None;
                        Self::spawn_statistics_monitor(
                            this.clone(),
                            generation,
                            launch.session,
                            cx,
                        );
                        view.login_dialog_open = false;
                        view.login_error = None;
                        view.session_connected = true;
                        view.session_label = "Backpage session verified".into();
                        view.game_pid = launch.pid;
                        view.client_window_visible = true;
                        view.hide_flash_after_startup = kind == ClientKind::Flash
                            && std::env::var("XENOMIT_FLASH_KEEP_VISIBLE").as_deref() != Ok("1");
                        #[cfg(windows)]
                        if let Some(pid) = launch.pid {
                            if kind != ClientKind::Flash {
                                view.client_window_visible = false;
                                Self::hide_client_window_when_ready(
                                    this.clone(),
                                    generation,
                                    pid,
                                    cx,
                                );
                            }
                        }
                        view.game_process = launch.child;
                        view.bridge = launch.bridge;
                        view.star_catalog = launch.star_catalog;
                        view.client_backend = backend;
                        view.client_flash_backend = flash_backend;
                        view.bridge_ready = false;
                        view.client_logs = launch.client_logs;
                        view.bridge_label = "Checking game connection…".into();
                        view.activity =
                            format!("{} connected. Waiting for game state.", kind.label());
                    }
                    Err(error) => {
                        view.resume_after_reload = false;
                        view.session_label = "Authentication failed".into();
                        view.login_error = Some(error.clone());
                        view.activity = error;
                    }
                }
                cx.notify();
            });
            if monitor.is_some() {
                // Shared-memory frames drive the map in the native backend;
                // the pipe only carries a cheap status request once per
                // second, never state reads. The legacy backend polls STATE
                // slowly instead — see spawn_live_monitor.
                Self::spawn_live_monitor(this.clone(), generation, cx);
                let mut polls: u32 = 0;
                loop {
                    cx.background_executor().timer(Duration::from_secs(1)).await;
                    polls = polls.wrapping_add(1);
                    // Read the bridge live so a backend switch rebinds this
                    // monitor without relaunching the client.
                    let bridge = this
                        .update(cx, |view, _cx| {
                            if view.connection_generation != generation || !view.session_connected {
                                return None;
                            }
                            view.bridge.clone()
                        })
                        .unwrap_or(None);
                    let Some(bridge) = bridge else {
                        break;
                    };
                    let ping_at = Instant::now();
                    let polled_bridge = bridge.clone();
                    let result = cx
                        .background_executor()
                        .spawn(async move { bridge.status().map_err(|error| error.to_string()) })
                        .await;
                    let ping_ms = ping_at.elapsed().as_millis() as u64;
                    // Boosters change on the order of minutes and the graphical
                    // clients observe them on the player thread, so poll slowly.
                    let boosters = if polls % 10 == 1
                        && result.as_ref().is_ok_and(|status| status.ready)
                    {
                        let client = Arc::clone(&polled_bridge);
                        cx.background_executor()
                            .spawn(async move {
                                client.feature_state().ok().and_then(|state| state.boosters)
                            })
                            .await
                    } else {
                        None
                    };
                    let stopped_tick = this
                        .update(cx, |view, _| {
                            if view.connection_generation != generation
                                || !view.session_connected
                                || view.engine.as_ref().is_some_and(|engine| engine.is_running())
                            {
                                return None;
                            }
                            let engine = view.engine.clone()?;
                            Some((
                                engine,
                                Arc::clone(&polled_bridge),
                                view.settings.module_config(),
                                result
                                    .as_ref()
                                    .ok()
                                    .filter(|status| status.ready)
                                    .and_then(|_| view.snapshot.clone()),
                            ))
                        })
                        .unwrap_or(None);
                    if let Some((engine, client, config, snapshot)) = stopped_tick {
                        let result = cx
                            .background_executor()
                            .spawn(async move {
                                engine.tick_stopped(client, &config, snapshot.as_ref())
                            })
                            .await;
                        if let Err(error) = result {
                            let _ = this.update(cx, |view, cx| {
                                if view.connection_generation == generation {
                                    view.activity = format!("Stopped behavior tick failed: {error}");
                                    cx.notify();
                                }
                            });
                        }
                    }
                    let still_active = this
                        .update(cx, |view, cx| {
                            if view.connection_generation != generation || !view.session_connected {
                                return false;
                            }
                            if !view.bridge.as_ref().is_some_and(|current| Arc::ptr_eq(current, &polled_bridge)) {
                                return true;
                            }
                            view.status_refreshes += 1;
                            view.bridge_ping_ms = Some(ping_ms);
                            if let Some(boosters) = boosters {
                                view.boosters = boosters;
                                view.boosters_at = Instant::now();
                            }
                            view.sample_process_usage();
                            match result {
                                Ok(status) => {
                                    if view.settings.client == ClientKind::Flash {
                                        view.live_fps = status
                                            .details
                                            .get("fps_milli")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0)
                                            as f32
                                            / 1000.0;
                                        view.flash_capture_us = status
                                            .details
                                            .get("capture_us_avg")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0)
                                            .min(u32::MAX as u64)
                                            as u32;
                                    }
                                    let was_ready = view.bridge_ready;
                                    view.bridge_ready = status.ready;
                                    view.bridge_label = if status.ready {
                                        "Game ready".into()
                                    } else if status.connected {
                                        "Game client connected; loading world".into()
                                    } else {
                                        "Game disconnected".into()
                                    };
                                    if status.ready && !was_ready {
                                        #[cfg(windows)]
                                        if kind == ClientKind::Flash
                                            && view.hide_flash_after_startup
                                        {
                                            view.hide_flash_after_startup = false;
                                            if let Some(pid) = view.game_pid {
                                                // Flash must activate visibly before Flex/game startup.
                                                // Hide only after the loaded hero/map proves startup completed.
                                                if set_client_window_visible(pid, false).is_ok() {
                                                    view.client_window_visible = false;
                                                }
                                            }
                                        }
                                        view.activity = format!(
                                            "{} is ready for the behavior engine.",
                                            kind.label()
                                        );
                                    }
                                }
                                Err(error) => {
                                    view.bridge_ready = false;
                                    view.bridge_label = format!("Game connection unavailable: {error}");
                                    view.snapshot = None;
                                }
                            }
                            if let Some(engine) = view.engine.clone() {
                                if view.behavior_running {
                                    // The worker reports its state every tick —
                                    // surface it so a silently idle or failed
                                    // module is visible instead of the stale
                                    // "accepted" message.
                                    if !engine.is_running() {
                                        view.behavior_running = false;
                                        if let Some(page) = view.module_pages.get("__plugins") {
                                            page.update(cx, |page, cx| page.set_locked(false, cx));
                                        }
                                    }
                                    if let Some(line) = engine.report() {
                                        view.activity = line;
                                    }
                                }
                            }
                            if view.resume_after_reload
                                && view.bridge_ready
                                && view.catalog_ready
                                && !view.behavior_starting
                            {
                                view.resume_after_reload = false;
                                view.run_behavior(cx, None);
                            }
                            for action in behavior::take_host_actions() {
                                match action {
                                    HostAction::Refresh => view.reload_client(cx),
                                    HostAction::RefreshIdle => view.reload_client_with_resume(Some(false), cx),
                                    HostAction::RequestRefresh => {}, // Runtime must approve cooperative requests.
                                    HostAction::Disconnect => {
                                        view.clear_safe_stop_request();
                                        view.stop_client(cx);
                                    }
                                    HostAction::Close => {
                                        view.clear_safe_stop_request();
                                        view.close_application(cx);
                                    }
                                    HostAction::Profile(name) => view.start_profile(name, cx),
                                    HostAction::TagPlayer { id, tag } => {
                                        view.settings.player_tags.insert(id, tag);
                                        view.save_settings();
                                    }
                                    HostAction::SelectTarget(_) => {}
                                    HostAction::AcceptUrgentQuest | HostAction::ClaimCalendar => {
                                        view.activity = "A requested game service is unavailable in this client session.".into();
                                    }
                                }
                            }
                            cx.notify();
                            true
                        })
                        .unwrap_or(false);
                    if !still_active {
                        break;
                    }
                }
            }
        })
        .detach();
    }
}
