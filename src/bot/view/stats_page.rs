//! Statistics monitor and the Statistics page.

use super::*;

impl BotView {
    pub(super) fn spawn_statistics_monitor(
        this: gpui_kit::WeakEntity<Self>,
        generation: u64,
        session: Session,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |_, cx| {
            loop {
                let stats = this
                    .update(cx, |view, _| {
                        if view.connection_generation == generation && view.session_connected {
                            view.statistics.clone()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(None);
                let Some(stats) = stats else {
                    break;
                };
                let session = session.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move { statistics::fetch_balances(session) })
                    .await;
                let active = this
                    .update(cx, |view, _| {
                        view.connection_generation == generation && view.session_connected
                    })
                    .unwrap_or(false);
                if !active {
                    break;
                }
                cx.background_executor()
                    .spawn(async move {
                        if let Ok(mut stats) = stats.lock() {
                            match result {
                                Ok(balances) => {
                                    stats.backpage_error = None;
                                    stats.reconcile(balances, "backpage", statistics::now_ms());
                                }
                                Err(error) => {
                                    stats.backpage_error =
                                        Some(format!("Backpage refresh failed: {error}"))
                                }
                            }
                        }
                    })
                    .await;
                let _ = this.update(cx, |_, cx| cx.notify());
                cx.background_executor()
                    .timer(Duration::from_secs(60))
                    .await;
            }
        })
        .detach();
    }

    pub(super) fn render_statistics(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut page = div()
            .id("statistics-page")
            .size_full()
            .overflow_y_scrollbar()
            .p(px(20.0))
            .flex()
            .flex_col()
            .gap(px(14.0));
        let Some(shared) = &self.statistics else {
            return page.child(label(
                "Connect an account to start recording statistics.",
                MUTED,
                13.0,
            ));
        };
        let stats = shared.lock().unwrap();
        let now = statistics::now_ms();

        let mut ranges = div()
            .flex()
            .gap(px(2.0))
            .p(px(3.0))
            .rounded_lg()
            .bg(rgb(PANEL_DARK))
            .border_1()
            .border_color(rgb(LINE_SOFT));
        for (title, range) in [
            ("1 hour", 3_600_000),
            ("24 hours", 24 * 3_600_000),
            ("7 days", 7 * 24 * 3_600_000),
            ("30 days", 30 * 24 * 3_600_000),
        ] {
            let selected = self.statistics_range == range;
            ranges = ranges.child(
                div()
                    .id(format!("stats-range-{range}"))
                    .px(px(12.0))
                    .py(px(5.0))
                    .rounded_md()
                    .bg(rgb(if selected { BLUE_TINT } else { PANEL_DARK }))
                    .hover(move |s| s.bg(rgb(if selected { BLUE_TINT } else { PANEL_HOVER })))
                    .cursor_pointer()
                    .text_size(px(12.0))
                    .font_weight(if selected {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(rgb(if selected { TEXT } else { MUTED }))
                    .child(title)
                    .on_click(cx.listener(move |view, _, _, cx| {
                        view.statistics_range = range;
                        cx.notify();
                    })),
            );
        }

        page = page.child(
            div()
                .flex()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(18.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(TEXT))
                                .child(format!("Asset history — {}", stats.account)),
                        )
                        .child(label(
                            &if let Some(at) = stats.last_reconciled {
                                format!(
                                    "Last balance update: {}s ago",
                                    now.saturating_sub(at) / 1000
                                )
                            } else {
                                "Waiting for the first balance update…".to_owned()
                            },
                            MUTED,
                            12.0,
                        )),
                )
                .child(ranges),
        );
        page = page
            .child(label("Gross rewards/hour: last hour of this connection. Balances refresh every minute.", MUTED, 12.0))
            .child(label("Rewards use game locale templates and protocol logs; other English reward messages are also recognized.", FAINT, 12.0));
        for error in [&stats.error, &stats.backpage_error].into_iter().flatten() {
            page = page.child(
                div()
                    .p(px(10.0))
                    .rounded_md()
                    .bg(rgb(RED_TINT))
                    .border_1()
                    .border_color(rgb(0x6b4046))
                    .child(label(error, 0xf0b9b9, 12.0)),
            );
        }
        let rates = stats.rates(now);
        let since = now.saturating_sub(self.statistics_range);
        for (index, title) in statistics::METRICS.iter().enumerate() {
            let mut points: Vec<(u64, f64)> = stats
                .events
                .iter()
                .filter_map(|event| event.balances[index].map(|value| (event.at, value)))
                .collect();
            let baseline = points.iter().rev().find(|(at, _)| *at < since).copied();
            points.retain(|(at, _)| *at >= since);
            if let Some((_, value)) = baseline {
                points.insert(0, (since, value));
            }
            let balance = stats.balances[index]
                .map(|v| format!("{v:.0}"))
                .unwrap_or_else(|| "Waiting…".into());
            let low = points.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
            let high = points
                .iter()
                .map(|(_, v)| *v)
                .fold(f64::NEG_INFINITY, f64::max);
            // Bound painted geometry for long histories, retaining the first,
            // last, minimum and maximum of each chronological bucket.
            if points.len() > 1200 {
                let bucket_size = points.len().div_ceil(300);
                let mut reduced = Vec::with_capacity(1200);
                for bucket in points.chunks(bucket_size) {
                    let minimum = bucket
                        .iter()
                        .enumerate()
                        .min_by(|a, b| a.1.1.total_cmp(&b.1.1))
                        .unwrap()
                        .0;
                    let maximum = bucket
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.1.total_cmp(&b.1.1))
                        .unwrap()
                        .0;
                    let mut indices = vec![0, minimum, maximum, bucket.len() - 1];
                    indices.sort_unstable();
                    indices.dedup();
                    for index in indices {
                        reduced.push(bucket[index]);
                    }
                }
                points = reduced;
            }
            let delta_value = points.first().zip(points.last()).map(|(a, b)| b.1 - a.1);
            let chart = canvas(
                move |_, _, _| (),
                move |bounds, _, window, _| {
                    use gpui_kit::{PathBuilder, point};
                    if points.is_empty() {
                        return;
                    }
                    let width = f32::from(bounds.size.width);
                    let height = f32::from(bounds.size.height);
                    let mut path = PathBuilder::stroke(px(1.5));
                    let mut area = PathBuilder::fill();
                    // Step plots preserve discrete rewards/spending rather than
                    // suggesting balances changed continuously between samples.
                    let mut last_y = None;
                    for (at, value) in &points {
                        let x = bounds.origin.x
                            + px(width * at.saturating_sub(since) as f32
                                / (now - since).max(1) as f32);
                        let normalized = if high == low {
                            0.5
                        } else {
                            (value - low) / (high - low)
                        };
                        let y =
                            bounds.origin.y + px(8.0 + (height - 16.0) * (1.0 - normalized as f32));
                        if let Some(previous) = last_y {
                            path.line_to(point(x, previous));
                            path.line_to(point(x, y));
                            area.line_to(point(x, previous));
                            area.line_to(point(x, y));
                        } else {
                            path.move_to(point(x, y));
                            area.move_to(point(x, y));
                        }
                        last_y = Some(y);
                    }
                    let bottom = bounds.origin.y + bounds.size.height;
                    if let Some(y) = last_y {
                        path.line_to(point(bounds.origin.x + bounds.size.width, y));
                        area.line_to(point(bounds.origin.x + bounds.size.width, y));
                    }
                    area.line_to(point(bounds.origin.x + bounds.size.width, bottom));
                    area.line_to(point(bounds.origin.x, bottom));
                    area.close();
                    if let Ok(area) = area.build() {
                        window.paint_path(area, rgb(BLUE).alpha(0.12));
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, rgb(BLUE));
                    }
                },
            )
            .w_full()
            .h(px(110.0))
            .flex_shrink_0();
            let mut card = card()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .p(px(14.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(rgb(TEXT))
                                        .child(title.to_string()),
                                )
                                .child(
                                    div()
                                        .font_family("Consolas")
                                        .text_size(px(15.0))
                                        .text_color(rgb(TEXT))
                                        .child(balance),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .when_some(delta_value, |this, delta| {
                                    this.child(
                                        div()
                                            .px(px(8.0))
                                            .py(px(3.0))
                                            .rounded_full()
                                            .bg(rgb(if delta >= 0.0 {
                                                GREEN_TINT
                                            } else {
                                                RED_TINT
                                            }))
                                            .text_size(px(11.5))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(if delta >= 0.0 {
                                                0x7fd6a4
                                            } else {
                                                0xe89c9c
                                            }))
                                            .child(format!("{delta:+.0}")),
                                    )
                                })
                                .child(
                                    div()
                                        .font_family("Consolas")
                                        .text_size(px(12.0))
                                        .text_color(rgb(MUTED))
                                        .child(format!("{:.0}/hour", rates[index])),
                                ),
                        ),
                );
            if low.is_finite() {
                card = card.child(label(
                    &format!("Balance scale: {low:.0} – {high:.0}"),
                    FAINT,
                    11.0,
                ));
            } else {
                card = card.child(label("Waiting for balance samples", MUTED, 12.0));
            }
            page = page.child(card.child(chart).child(label(
                &format!("{} hours ago → now", self.statistics_range / 3_600_000),
                FAINT,
                11.0,
            )));
        }
        page = page.child(section("RECENT GAME LOGS"));
        let mut log_rows = div().flex().flex_col().gap(px(2.0));
        let mut has_logs = false;
        for event in stats
            .events
            .iter()
            .rev()
            .filter(|e| e.message.is_some())
            .take(50)
        {
            has_logs = true;
            log_rows = log_rows.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .px(px(10.0))
                    .py(px(5.0))
                    .rounded_md()
                    .child(
                        div()
                            .w(px(64.0))
                            .flex_shrink_0()
                            .font_family("Consolas")
                            .text_size(px(11.0))
                            .text_color(rgb(FAINT))
                            .child(format!("{}s ago", now.saturating_sub(event.at) / 1000)),
                    )
                    .child(label(event.message.as_deref().unwrap(), TEXT, 12.0)),
            );
        }
        if !has_logs {
            log_rows = log_rows.child(label("No game log events recorded yet.", FAINT, 12.0));
        }
        page.child(log_rows)
    }
}
