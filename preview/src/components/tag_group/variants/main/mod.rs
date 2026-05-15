use dioxus::prelude::*;

use dioxus_primitives::tag_group::SelectionMode;
use super::super::component::*;
use std::collections::HashSet;

#[component]
pub fn Demo() -> Element {
    let items = [
        ("bug", "bug"),
        ("feature", "feature"),
        ("core", "core"),
        ("desktop", "desktop"),
        ("example", "example"),
        ("duplicate", "duplicate"),
    ]
    .map(|(key, label)| {
        rsx! {
            span { key: "{key}", "{label}" }
        }
    })
    .to_vec();

    let mut selected = use_signal(|| HashSet::from(["bug".into()]));
    let selected_tags = use_memo(move || Some(selected()));

    rsx! {
        TagGroup {
            label: "Labels",
            items,
            selection_mode: SelectionMode::Multiple,
            disabled_tags: HashSet::from(["feature".into(), "example".into()]),
            selected_tags,
            on_selection_change: move |tags| selected.set(tags),
            allows_empty_selection: false,
            allows_removing: true,
        }
    }
}
