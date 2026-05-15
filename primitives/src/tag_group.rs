//! Defines the [`TagGroup`] component and its sub-components.

use dioxus::prelude::*;

use crate::focus::{use_focus_controlled_item_disabled, use_focus_provider, FocusState};
use crate::list_data::element_key;
use crate::{use_controlled, use_unique_id};

use std::collections::HashSet;

/// The type of selection that is allowed in [`TagGroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// No selection (`aria-selected` is not set).
    #[default]
    None,
    /// At most one tag may be selected.
    Single,
    /// Any number of tags may be selected.
    Multiple,
}

/// Context provided by [`TagGroup`] to its descendants.
/// Use `use_context::<TagGroupCtx>()` to access list-level operations.
#[derive(Clone, Copy)]
pub struct TagGroupCtx {
    // State
    list_items: Signal<Vec<Element>>,
    // ID of the element that labels this group
    labeled_by: Signal<Option<String>>,
    selection_mode: SelectionMode,
    selected_tags: Memo<HashSet<String>>,
    on_selection_change: Callback<HashSet<String>>,
    disabled_tags: ReadSignal<HashSet<String>>,

    // Configuration
    focus: FocusState,
    group_disabled: ReadSignal<bool>,
    allows_empty_selection: ReadSignal<bool>,
    escape_clears_selection: ReadSignal<bool>,
    allows_removing: ReadSignal<bool>,
    render_empty_state: Callback<(), Element>,
}

impl TagGroupCtx {
    /// Returns whether tags in this group show a remove control and can be deleted.
    pub fn is_removable(&self) -> bool {
        (self.allows_removing)()
    }

    /// Returns the stable key for the tag at `index`
    pub fn item_key(&self, index: usize) -> String {
        (self.list_items)()
            .get(index)
            .map(|element| element_key(element, index))
            .unwrap_or_else(|| index.to_string())
    }

    fn is_tag_disabled(&self, key: &str) -> bool {
        (self.group_disabled)() || (self.disabled_tags)().contains(key)
    }

    fn is_tag_selected(&self, key: &str) -> bool {
        (self.selected_tags)().contains(key)
    }

    fn toggle_tag(&self, key: String) {
        let allows_empty_selection = (self.allows_empty_selection)();
        let mut next = (self.selected_tags)().clone();
        match self.selection_mode {
            SelectionMode::None => {
                return;
            }
            SelectionMode::Single => {
                if !next.contains(&key) {
                    next.clear();
                    next.insert(key);
                } else if allows_empty_selection || next.len() > 1 {
                    next.clear();
                }
            }
            SelectionMode::Multiple => {
                if !next.contains(&key) {
                    next.insert(key);
                } else if allows_empty_selection || next.len() > 1 {
                    next.remove(&key);
                }
            }
        }

        self.on_selection_change.call(next);
    }

    fn clear_selection(&self) {
        match self.selection_mode {
            SelectionMode::None => {}
            SelectionMode::Single | SelectionMode::Multiple => {
                if (self.escape_clears_selection)() {
                    self.on_selection_change.call(HashSet::new());
                }
            }
        }
    }

    /// Removes tags with the given keys from the list and clears them from the current selection.
    pub fn remove_tags(&mut self, keys: HashSet<String>) {
        if keys.is_empty() {
            return;
        }

        let mut list = (self.list_items)();
        let mut indices: Vec<usize> = list
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                keys.contains(&element_key(element, index)).then_some(index)
            })
            .collect();
        indices.sort_unstable_by(|a, b| b.cmp(a));
        for index in indices {
            let _ = list.remove(index);
        }
        self.list_items.set(list);

        let mut selected = (self.selected_tags)().clone();
        for key in &keys {
            selected.remove(key);
        }
        if selected != (self.selected_tags)() {
            self.on_selection_change.call(selected);
        }
    }

    fn keyboard_remove(&mut self) {
        if !(self.allows_removing)()
            || self.selection_mode == SelectionMode::None
            || (self.selected_tags)().is_empty()
        {
            return;
        }
        self.remove_tags((self.selected_tags)());
    }
}

/// The props for the [`TagGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TagGroupProps {
    /// Optional label above the tag group.
    #[props(default)]
    pub label: Option<String>,

    /// Tag content to render inside [`TagList`].
    pub items: Vec<Element>,

    /// The type of selection that is allowed in the group.
    #[props(default)]
    pub selection_mode: SelectionMode,

    /// The currently selected tag keys (controlled). `None` means uncontrolled.
    #[props(default)]
    pub selected_tags: ReadSignal<Option<HashSet<String>>>,

    /// The initial selected tag keys (uncontrolled).
    #[props(default)]
    pub default_selected_tags: HashSet<String>,

    /// Handler that is called when the selection changes.
    #[props(default)]
    pub on_selection_change: Callback<HashSet<String>>,

    /// The tag keys that are disabled. These items cannot be selected, focused, or otherwise interacted with.
    #[props(default = ReadSignal::new(Signal::new(HashSet::new())))]
    pub disabled_tags: ReadSignal<HashSet<String>>,

    /// Whether the tag group is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the collection allows empty selection.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub allows_empty_selection: ReadSignal<bool>,

    /// Whether pressing the ESC key should clear selection in the TagGroup or not.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub escape_clears_selection: ReadSignal<bool>,

    /// Shows a remove control on tags and enables Delete/Backspace removal.
    #[props(default)]
    pub allows_removing: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Renders content when [`TagGroupProps::items`] is empty.
    #[props(default = Callback::new(|_| rsx! { div { "No tags" }}))]
    pub render_empty_state: Callback<(), Element>,

    /// Additional attributes to apply to the tag group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the tag group component. Defaults to [`TagList`].
    #[props(default)]
    pub children: Option<Element>,
}

/// # TagGroup
///
/// A focusable group of tags with optional selection and removal.
/// Pass tag content via [`TagGroupProps::items`] (set a vnode `key` on each item)
/// and render them with [`TagList`] / [`Tag`], or customize the list in `children`.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::tag_group::{TagGroup, SelectionMode};
/// use std::collections::HashSet;
///
/// #[component]
/// fn Demo() -> Element {
///     let items = ["bug", "feature"]
///         .into_iter()
///         .map(|label| rsx! {
///             span { key: "{label}", {label} }
///         })
///         .collect::<Vec<_>>();
///
///     let mut selected = use_signal(|| HashSet::from(["bug".to_string()]));
///     let selected_tags = use_memo(move || Some(selected()));
///
///     rsx! {
///         TagGroup {
///             label: "Labels",
///             items,
///             selection_mode: SelectionMode::Multiple,
///             selected_tags,
///             on_selection_change: move |tags| selected.set(tags),
///             allows_removing: true,
///         }
///     }
/// }
/// ```
#[component]
pub fn TagGroup(props: TagGroupProps) -> Element {
    let label_id = use_unique_id();
    let mut labeled_by = use_signal(|| None);
    labeled_by.set(props.label.as_ref().map(|_| label_id()));

    let (selected_tags, set_selected_tags) = use_controlled(
        props.selected_tags,
        props.default_selected_tags.clone(),
        props.on_selection_change,
    );

    let list_items = use_signal(|| props.items.clone());
    let focus = use_focus_provider(props.roving_loop);

    use_context_provider(|| TagGroupCtx {
        labeled_by,
        selection_mode: props.selection_mode,
        selected_tags,
        on_selection_change: set_selected_tags,
        disabled_tags: props.disabled_tags,
        list_items,
        focus,
        group_disabled: props.disabled,
        allows_empty_selection: props.allows_empty_selection,
        escape_clears_selection: props.escape_clears_selection,
        allows_removing: props.allows_removing,
        render_empty_state: props.render_empty_state,
    });

    let children = props.children.unwrap_or_else(|| rsx! { TagList {} });

    rsx! {
        div {
            ..props.attributes,
            if let Some(label) = props.label {
                span {
                    id: label_id(),
                    {label}
                }
            }
            {children}
        }
    }
}

/// Data for rendering a tag in [`TagList`].
#[derive(Clone, PartialEq)]
pub struct TagListRenderItem {
    /// The current index of this tag.
    pub index: usize,
    /// The stable key for this tag.
    pub key: String,
    /// The rendered tag children.
    pub children: Element,
}

/// Returns render data for the current tags in [`TagGroup`].
pub fn use_tag_list_items() -> Vec<TagListRenderItem> {
    let ctx: TagGroupCtx = use_context();
    (ctx.list_items)()
        .into_iter()
        .enumerate()
        .map(|(index, children)| TagListRenderItem {
            index,
            key: ctx.item_key(index),
            children,
        })
        .collect()
}

/// The props for the [`TagList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TagListProps {
    /// Additional attributes to apply to the tag list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the tag list component. Defaults to a [`Tag`] per item from [`TagGroup::items`].
    #[props(default)]
    pub children: Option<Element>,
}

/// # TagList
///
/// The inner grid element for tags inside a [`TagGroup`].
/// Renders with one [`Tag`] per row by default. When [`TagGroupProps::items`] is empty, shows
/// [`TagGroupProps::render_empty_state`] instead of the list.
///
/// This must be used inside a [`TagGroup`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::tag_group::{Tag, TagGroup, TagList, use_tag_list_items};
///
/// #[component]
/// fn Demo() -> Element {
///     let items = ["bug", "feature"]
///         .into_iter()
///         .map(|label| rsx! {
///             span { key: "{label}", {label} }
///         })
///         .collect::<Vec<_>>();
///
///     rsx! {
///         TagGroup {
///             label: "Labels",
///             items,
///             TagList {
///                 for item in use_tag_list_items() {
///                     Tag {
///                         key: "{item.key}",
///                         index: item.index,
///                         {item.children}
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn TagList(props: TagListProps) -> Element {
    let ctx = use_context::<TagGroupCtx>();
    let is_empty = (ctx.list_items)().is_empty();

    let children = props.children.unwrap_or_else(|| {
        rsx! {
            for item in use_tag_list_items() {
                Tag {
                    key: "{item.key}",
                    index: item.index,
                    {item.children}
                }
            }
        }
    });

    rsx! {
        div {
            role: "grid",
            aria_labelledby: ctx.labeled_by,
            tabindex: "-1",
            aria_multiselectable: if ctx.selection_mode == SelectionMode::Multiple { "true" },
            aria_colcount: "1",
            ..props.attributes,
            if is_empty {
                {ctx.render_empty_state.call(())}
            } else {
                {children}
            }
        }
    }
}

/// The props for the [`Tag`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    /// The index of the tag in the list.
    pub index: usize,

    /// Whether this tag is disabled in addition to group-level [`TagGroupProps::disabled_tags`].
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the tag element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the tag component.
    pub children: Element,
}

/// # Tag
///
/// A single tag row inside [`TagList`].
/// Handles focus, selection (Space/Enter), arrow-key navigation, and removal
/// (Delete/Backspace when [`TagGroupProps::allows_removing`] is enabled).
///
/// Pass the list index from [`use_tag_list_items`] or from your own `enumerate()`.
/// This must be used within a [`TagGroup`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::tag_group::{Tag, TagGroup, TagList, use_tag_list_items};
///
/// #[component]
/// fn Demo() -> Element {
///     let items = [rsx! { span { key: "{0}", "bug" } }].to_vec();
///
///     rsx! {
///         TagGroup {
///             items,
///             TagList {
///                 for item in use_tag_list_items() {
///                     Tag {
///                         key: "{item.key}",
///                         index: item.index,
///                         {item.children}
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Tag`] component defines the following data attributes you can use to control styling:
/// - `data-selected`: `true` when the tag is selected.
/// - `data-disabled`: `true` when the tag is disabled.
#[component]
pub fn Tag(props: TagProps) -> Element {
    let index = props.index;
    let mut ctx = use_context::<TagGroupCtx>();
    let tag_key = move || ctx.item_key(index);

    let tabindex = use_memo(move || {
        if !(ctx.focus.roving_loop)() {
            return "0";
        }
        if ctx.focus.recent_focus_or_default() == index {
            "0"
        } else {
            "-1"
        }
    });

    let is_selected = move || ctx.is_tag_selected(&tag_key());
    let is_disabled = move || ctx.is_tag_disabled(&tag_key()) || (props.disabled)();
    let index_signal = use_memo(move || index);
    let onmounted = use_focus_controlled_item_disabled(index_signal, is_disabled);

    let onkeydown = move |e: Event<KeyboardData>| {
        if is_disabled() {
            return;
        }
        let event_key = e.key();
        let mut prevent_default = false;

        match event_key {
            Key::Escape => {
                ctx.clear_selection();
                prevent_default = true;
            }
            Key::Character(s) if s == " " => {
                ctx.toggle_tag(tag_key());
                prevent_default = true;
            }
            Key::Enter => {
                ctx.toggle_tag(tag_key());
                prevent_default = true;
            }
            Key::Backspace | Key::Delete => {
                ctx.keyboard_remove();
                prevent_default = true;
            }
            Key::ArrowUp | Key::ArrowLeft => {
                ctx.focus.focus_prev();
                prevent_default = true;
            }
            Key::ArrowDown | Key::ArrowRight => {
                ctx.focus.focus_next();
                prevent_default = true;
            }
            Key::Home => {
                ctx.focus.focus_first();
                prevent_default = true;
            }
            Key::End => {
                ctx.focus.focus_last();
                prevent_default = true;
            }
            _ => {}
        }

        if prevent_default {
            e.prevent_default();
        }
    };

    rsx! {
        div {
            role: "row",
            tabindex,
            aria_selected: if ctx.selection_mode != SelectionMode::None { is_selected() },
            aria_disabled: is_disabled(),
            "data-selected": is_selected(),
            "data-disabled": is_disabled(),
            onmounted,
            onfocus: move |_| ctx.focus.set_focus(Some(index)),
            onkeydown,
            onclick: move |_| {
                if !is_disabled() {
                    ctx.toggle_tag(tag_key());
                }
            },
            ..props.attributes,
            div {
                role: "gridcell",
                aria_colindex: "1",
                display: "contents",
                {props.children}
            }
        }
    }
}
