use super::*;

impl Render for ModuleSettingsPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.maps_dirty {
            for (key, select) in &self.map_selects {
                let selected = setting_value(&self.values, key).unwrap_or(&Value::Null);
                let choices = map_choices(&self.maps, selected);
                let selected = resolved_map_value(&self.maps, selected);
                select.update(cx, |state, cx| {
                    state.set_items(choices, window, cx);
                    state.set_selected_value(&selected, window, cx);
                });
            }
            self.maps_dirty = false;
        }
        // One scrolling column. Sections stack the way the reference dialog
        // does: a disclosure header, then one compact line per setting.
        let mut rows = div()
            .id(format!("settings-page-{}", self.descriptor.id))
            .size_full()
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .overflow_y_scrollbar()
            .px(px(8.0))
            .py(px(4.0));
        if !self.descriptor.description.is_empty() {
            rows = rows.child(div().pb(px(4.0)).child(label(
                &self.descriptor.description,
                MUTED,
                12.0,
            )));
        }
        let fields = self
            .descriptor
            .settings
            .iter()
            .filter(|field| {
                self.visible_rules.as_ref().is_none_or(|visible| {
                    field
                        .key
                        .split_once('.')
                        .is_none_or(|(rule, _)| visible.contains(rule))
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut previous_group = String::new();
        for field in &fields {
            if previous_group != field.group {
                previous_group = field.group.clone();
                let open = !self.collapsed.contains(&field.group);
                let group = field.group.clone();
                rows = rows.child(
                    div()
                        .id(format!("settings-group-{}", group))
                        .mt(px(6.0))
                        .h(px(22.0))
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .border_b_1()
                        .border_color(rgb(LINE))
                        .cursor_pointer()
                        .on_click(cx.listener(move |page, _, _, cx| {
                            if !page.collapsed.remove(&group) {
                                page.collapsed.insert(group.clone());
                            }
                            cx.notify();
                        }))
                        .child(
                            div()
                                .w(px(10.0))
                                .text_size(px(12.0))
                                .text_color(rgb(MUTED))
                                .child(if open { "▾" } else { "▸" }),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(TEXT))
                                .child(field.group.clone()),
                        ),
                );
                if !open {
                    continue;
                }
            } else if self.collapsed.contains(&field.group) {
                continue;
            }
            let value = setting_value(&self.values, &field.key).unwrap_or(&field.default);
            rows = rows.child(self.render_field(field, value, cx));
        }
        rows
    }
}
