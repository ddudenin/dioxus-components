//! Defines the [`TagGroup`] and [`TagGroupMulti`] components and their sub-components.

use dioxus::prelude::*;

use crate::{
    focus::{
        use_focus_controlled_item_disabled, use_focus_entry_disabled, use_focus_provider,
        FocusState,
    },
    selectable::SelectionMode,
    selection::{option_text_value, remove_option, sync_option, OptionState, RcPartialEqValue},
    use_controlled, use_effect_cleanup, use_id_or, use_unique_id,
};

/// Selection and focus state for a tag group.
#[derive(Clone, Copy)]
pub(crate) struct TagSelectableContext {
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    clear_selection: Callback<()>,
    selection_mode: SelectionMode,
    options: Signal<Vec<OptionState>>,
    focus: FocusState,
    disabled: ReadSignal<bool>,
    selectable: ReadSignal<bool>,
    allow_empty_selection: ReadSignal<bool>,
}

/// Context provided by [`TagGroup`] / [`TagGroupMulti`] to descendants.
#[derive(Clone, Copy)]
pub struct TagGroupCtx {
    labeled_by: Signal<Option<String>>,
    allows_removing: ReadSignal<bool>,
    escape_clears_selection: ReadSignal<bool>,
    disabled_values: ReadSignal<Vec<RcPartialEqValue>>,
    removed: Signal<Vec<RcPartialEqValue>>,
    render_empty_state: Callback<(), Element>,
}

#[derive(Clone)]
struct TagOptionCtx {
    value: RcPartialEqValue,
    index: ReadSignal<usize>,
}

struct TagGroupSharedProps<T: Clone + PartialEq + 'static> {
    label: Option<String>,
    disabled: ReadSignal<bool>,
    selectable: ReadSignal<bool>,
    disabled_values: ReadSignal<Vec<T>>,
    allow_empty_selection: ReadSignal<bool>,
    escape_clears_selection: ReadSignal<bool>,
    allows_removing: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    render_empty_state: Callback<(), Element>,
    attributes: Vec<Attribute>,
    children: Element,
}

struct TagGroupSelection {
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    clear_selection: Callback<()>,
    selection_mode: SelectionMode,
}

impl<T: Clone + PartialEq + 'static> TagGroupSharedProps<T> {
    fn from_single(props: &TagGroupProps<T>) -> Self {
        Self {
            label: props.label.clone(),
            disabled: props.disabled,
            selectable: props.selectable,
            disabled_values: props.disabled_values,
            allow_empty_selection: props.allow_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            allows_removing: props.allows_removing,
            roving_loop: props.roving_loop,
            render_empty_state: props.render_empty_state,
            attributes: props.attributes.clone(),
            children: props.children.clone(),
        }
    }

    fn from_multi(props: &TagGroupMultiProps<T>) -> Self {
        Self {
            label: props.label.clone(),
            disabled: props.disabled,
            selectable: props.selectable,
            disabled_values: props.disabled_values,
            allow_empty_selection: props.allow_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            allows_removing: props.allows_removing,
            roving_loop: props.roving_loop,
            render_empty_state: props.render_empty_state,
            attributes: props.attributes.clone(),
            children: props.children.clone(),
        }
    }
}

impl TagGroupCtx {
    /// Whether tags in this group show a remove control and can be deleted.
    pub fn is_removable(&self) -> bool {
        (self.allows_removing)()
    }

    fn remove_value(&mut self, selectable: TagSelectableContext, value: RcPartialEqValue) {
        let mut removed = self.removed.write();
        if removed.iter().any(|v| v == &value) {
            return;
        }
        removed.push(value.clone());
        drop(removed);

        if selectable.is_selected(&value) {
            match selectable.selection_mode {
                SelectionMode::Single => selectable.clear_selection.call(()),
                SelectionMode::Multiple => selectable.set_value.call(value),
            }
        }
    }

    fn is_removed(&self, value: &RcPartialEqValue) -> bool {
        self.removed.read().iter().any(|v| v == value)
    }

    fn is_empty(&self, selectable: TagSelectableContext) -> bool {
        selectable
            .options
            .read()
            .iter()
            .all(|option| self.is_removed(&option.value))
    }
}

impl TagSelectableContext {
    fn is_selected(&self, value: &RcPartialEqValue) -> bool {
        self.values.read().iter().any(|v| v == value)
    }

    fn toggle_value(&self, value: RcPartialEqValue) {
        if !(self.selectable)() {
            return;
        }

        let deselecting = self.is_selected(&value);
        if !deselecting {
            self.set_value.call(value);
            return;
        }

        let can_clear = match self.selection_mode {
            SelectionMode::Single => (self.allow_empty_selection)(),
            SelectionMode::Multiple => {
                (self.allow_empty_selection)() || self.values.read().len() > 1
            }
        };

        if can_clear {
            match self.selection_mode {
                SelectionMode::Single => self.clear_selection.call(()),
                SelectionMode::Multiple => self.set_value.call(value),
            }
        }
    }

    fn keyboard_remove_values(&self, focused: RcPartialEqValue) -> Vec<RcPartialEqValue> {
        if self.selection_mode == SelectionMode::Multiple && !self.values.read().is_empty() {
            self.values.read().clone()
        } else {
            vec![focused]
        }
    }
}

/// Props for [`TagGroup`] (single selection).
#[derive(Props, Clone, PartialEq)]
pub struct TagGroupProps<T: Clone + PartialEq + 'static = String> {
    /// Controlled selected value. `None` in the signal means no tag is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// Initial value when uncontrolled.
    #[props(default)]
    pub default_value: Option<T>,

    /// Called when the selected value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Optional visible label for the group, referenced by the tag list via `aria-labelledby`.
    #[props(default)]
    pub label: Option<String>,

    /// Whether the entire tag group is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether tags can be selected. When `false`, tags remain focusable but not selectable.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub selectable: ReadSignal<bool>,

    /// Values that cannot be selected or focused.
    #[props(default = ReadSignal::new(Signal::new(Vec::new())))]
    pub disabled_values: ReadSignal<Vec<T>>,

    /// Whether clicking or pressing Space/Enter on the selected tag clears the selection.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub allow_empty_selection: ReadSignal<bool>,

    /// Whether pressing Escape clears the current selection.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub escape_clears_selection: ReadSignal<bool>,

    /// Whether tags can be removed via [`TagRemoveButton`] or Delete/Backspace.
    #[props(default)]
    pub allows_removing: ReadSignal<bool>,

    /// Whether keyboard focus loops from the last tag to the first and vice versa.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Content rendered inside [`TagList`] when there are no tags.
    #[props(default = Callback::new(|_| rsx! { div { "No tags" } }))]
    pub render_empty_state: Callback<(), Element>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the tag group, typically a [`TagList`] with [`TagOption`] children.
    pub children: Element,
}

/// Props for [`TagGroupMulti`] (multiple selection).
#[derive(Props, Clone, PartialEq)]
pub struct TagGroupMultiProps<T: Clone + PartialEq + 'static = String> {
    /// Controlled selected values.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<T>>>,

    /// Initial values when uncontrolled.
    #[props(default)]
    pub default_values: Vec<T>,

    /// Called when the selected values change.
    #[props(default)]
    pub on_values_change: Callback<Vec<T>>,

    /// Optional visible label for the group, referenced by the tag list via `aria-labelledby`.
    #[props(default)]
    pub label: Option<String>,

    /// Whether the entire tag group is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether tags can be selected. When `false`, tags remain focusable but not selectable.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub selectable: ReadSignal<bool>,

    /// Values that cannot be selected or focused.
    #[props(default = ReadSignal::new(Signal::new(Vec::new())))]
    pub disabled_values: ReadSignal<Vec<T>>,

    /// Whether clicking or pressing Space/Enter on a selected tag deselects it.
    /// When `false`, the last remaining selected tag cannot be deselected.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub allow_empty_selection: ReadSignal<bool>,

    /// Whether pressing Escape clears the current selection.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub escape_clears_selection: ReadSignal<bool>,

    /// Whether tags can be removed via [`TagRemoveButton`] or Delete/Backspace.
    #[props(default)]
    pub allows_removing: ReadSignal<bool>,

    /// Whether keyboard focus loops from the last tag to the first and vice versa.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Content rendered inside [`TagList`] when there are no tags.
    #[props(default = Callback::new(|_| rsx! { div { "No tags" } }))]
    pub render_empty_state: Callback<(), Element>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the tag group, typically a [`TagList`] with [`TagOption`] children.
    pub children: Element,
}

/// # TagGroup
///
/// A focusable group of tags with single selection.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::tag_group::{TagGroup, TagList, TagOption};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         TagGroup::<&'static str> {
///             label: "Labels",
///             default_value: Some("bug"),
///             TagList {
///                 TagOption::<&'static str> { index: 0usize, value: "bug", "bug" }
///                 TagOption::<&'static str> { index: 1usize, value: "feature", disabled: true, "feature" }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn TagGroup<T: Clone + PartialEq + 'static>(props: TagGroupProps<T>) -> Element {
    let mut internal_value: Signal<Option<T>> = use_signal(|| props.default_value.clone());
    let value = use_memo(move || match props.value {
        Some(value) => value.cloned(),
        None => internal_value.cloned(),
    });
    let values = use_memo(move || value().map(RcPartialEqValue::new).into_iter().collect());
    let on_change = props.on_value_change;
    let set_value = use_callback(move |incoming: RcPartialEqValue| {
        let value = incoming
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("TagGroup and TagOption value types must match"))
            .clone();
        internal_value.set(Some(value.clone()));
        on_change.call(Some(value));
    });
    let clear_selection = use_callback(move |_| {
        internal_value.set(None);
        on_change.call(None);
    });

    tag_group_inner(
        TagGroupSharedProps::from_single(&props),
        TagGroupSelection {
            values,
            set_value,
            clear_selection,
            selection_mode: SelectionMode::Single,
        },
    )
}

/// # TagGroupMulti
///
/// A focusable group of tags with multiple selection.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::tag_group::{TagGroupMulti, TagList, TagOption};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         TagGroupMulti::<&'static str> {
///             label: "Labels",
///             default_values: vec!["bug"],
///             TagList {
///                 TagOption::<&'static str> { index: 0usize, value: "bug", "bug" }
///                 TagOption::<&'static str> { index: 1usize, value: "feature", "feature" }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn TagGroupMulti<T: Clone + PartialEq + 'static>(props: TagGroupMultiProps<T>) -> Element {
    let (multi_values, set_multi_internal) = use_controlled(
        props.values,
        props.default_values.clone(),
        props.on_values_change,
    );

    let values = use_memo(move || {
        multi_values()
            .into_iter()
            .map(RcPartialEqValue::new)
            .collect()
    });
    let set_value = use_callback(move |value: RcPartialEqValue| {
        let value_t = value
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("TagGroupMulti and TagOption value types must match"))
            .clone();
        let mut current = multi_values();
        if let Some(pos) = current.iter().position(|v| v == &value_t) {
            current.remove(pos);
        } else {
            current.push(value_t);
        }
        set_multi_internal.call(current);
    });
    let clear_selection = use_callback(move |_| {
        set_multi_internal.call(Vec::new());
    });

    tag_group_inner(
        TagGroupSharedProps::from_multi(&props),
        TagGroupSelection {
            values,
            set_value,
            clear_selection,
            selection_mode: SelectionMode::Multiple,
        },
    )
}

fn tag_group_inner<T: Clone + PartialEq + 'static>(
    shared: TagGroupSharedProps<T>,
    selection: TagGroupSelection,
) -> Element {
    let TagGroupSharedProps {
        label,
        disabled,
        selectable,
        disabled_values,
        allow_empty_selection,
        escape_clears_selection,
        allows_removing,
        roving_loop,
        render_empty_state,
        attributes,
        children,
    } = shared;
    let TagGroupSelection {
        values,
        set_value,
        clear_selection,
        selection_mode,
    } = selection;

    let label_id = use_unique_id();
    let disabled_values = use_memo(move || {
        disabled_values
            .read()
            .iter()
            .cloned()
            .map(RcPartialEqValue::new)
            .collect::<Vec<_>>()
    });
    let disabled_values = ReadSignal::new(disabled_values);

    let options: Signal<Vec<OptionState>> = use_signal(Vec::default);
    let focus = use_focus_provider(roving_loop);
    let removed: Signal<Vec<RcPartialEqValue>> = use_signal(Vec::default);

    use_context_provider(|| TagSelectableContext {
        values,
        set_value,
        clear_selection,
        selection_mode,
        options,
        focus,
        disabled,
        selectable,
        allow_empty_selection,
    });

    let mut ctx = TagGroupCtx {
        labeled_by: Signal::new(None),
        allows_removing,
        escape_clears_selection,
        disabled_values,
        removed,
        render_empty_state,
    };
    ctx.labeled_by.set(label.as_ref().map(|_| label_id()));
    use_context_provider(|| ctx);

    rsx! {
        div {
            ..attributes,
            if let Some(label) = label {
                span {
                    id: label_id(),
                    {label}
                }
            }
            {children}
        }
    }
}

/// The props for the [`TagList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TagListProps {
    /// Additional attributes for the grid element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// [`TagOption`] children rendered as rows in the grid.
    pub children: Element,
}

/// Grid container for [`TagOption`] children.
#[component]
pub fn TagList(props: TagListProps) -> Element {
    let ctx = use_context::<TagGroupCtx>();
    let selectable = use_context::<TagSelectableContext>();
    let mut mounted = use_signal(|| false);
    use_effect(move || mounted.set(true));
    let is_empty = use_memo(move || mounted() && ctx.is_empty(selectable));

    rsx! {
        div {
            role: "grid",
            aria_labelledby: ctx.labeled_by,
            tabindex: "-1",
            aria_multiselectable: if selectable.selection_mode == SelectionMode::Multiple
                && (selectable.selectable)()
            {
                "true"
            },
            aria_colcount: "1",
            ..props.attributes,
            {props.children}
            if is_empty() {
                {ctx.render_empty_state.call(())}
            }
        }
    }
}

/// Props for [`TagOption`].
#[derive(Props, Clone, PartialEq)]
pub struct TagOptionProps<T: Clone + PartialEq + 'static = String> {
    /// Programmatic value for this tag (selection and removal).
    pub value: ReadSignal<T>,

    /// Text used for the remove button label when no [`TagOptionProps::text_value`] is set.
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Index for focus order and `aria-rowindex`.
    pub index: ReadSignal<usize>,

    /// Optional ID for the tag row element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Whether this tag is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes for the tag row element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The tag label and optional [`TagRemoveButton`].
    pub children: Element,
}

fn tag_option_on_keydown(
    e: Event<KeyboardData>,
    mut ctx: TagGroupCtx,
    mut selectable: TagSelectableContext,
    value: RcPartialEqValue,
    is_disabled: bool,
    removable: bool,
) {
    if is_disabled {
        return;
    }

    let key = e.key();
    let mut prevent_default = false;

    match key {
        Key::Escape if (ctx.escape_clears_selection)() => {
            selectable.clear_selection.call(());
            prevent_default = true;
        }
        Key::Character(s) if s == " " => {
            selectable.toggle_value(value.clone());
            prevent_default = true;
        }
        Key::Enter => {
            selectable.toggle_value(value.clone());
            prevent_default = true;
        }
        Key::Backspace | Key::Delete if removable && (selectable.selectable)() => {
            for value in selectable.keyboard_remove_values(value) {
                ctx.remove_value(selectable, value);
            }
            prevent_default = true;
        }
        Key::ArrowUp | Key::ArrowLeft => {
            selectable.focus.focus_prev();
            prevent_default = true;
        }
        Key::ArrowDown | Key::ArrowRight => {
            selectable.focus.focus_next();
            prevent_default = true;
        }
        Key::Home => {
            selectable.focus.focus_first();
            prevent_default = true;
        }
        Key::End => {
            selectable.focus.focus_last();
            prevent_default = true;
        }
        _ => {}
    }

    if prevent_default {
        e.prevent_default();
    }
}

/// A single tag inside [`TagList`]. Must be used within [`TagGroup`] or [`TagGroupMulti`].
#[component]
pub fn TagOption<T: Clone + PartialEq + 'static>(props: TagOptionProps<T>) -> Element {
    let ctx: TagGroupCtx = use_context();
    let mut selectable = use_context::<TagSelectableContext>();
    let index = props.index;
    let option_disabled = props.disabled;
    let text_value_signal = props.text_value;
    let option_value = props.value;
    let value = use_memo(move || RcPartialEqValue::new(option_value.cloned()));
    let is_removed = use_memo(move || ctx.is_removed(&value()));

    let disabled = {
        let root_disabled = selectable.disabled;
        let group_disabled_values = ctx.disabled_values;
        use_memo(move || {
            root_disabled.cloned()
                || option_disabled.cloned()
                || group_disabled_values.read().iter().any(|v| v == &value())
        })
    };

    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, props.id);
    let mut previous_id: Signal<Option<String>> = use_signal(|| None);
    let text_value = use_memo(move || {
        option_text_value(&*option_value.read(), text_value_signal(), "TagOption")
    });

    use_effect(move || {
        let option_id = id();
        let stale_id = previous_id
            .peek()
            .as_ref()
            .filter(|stale_id| *stale_id != &option_id)
            .cloned();
        if let Some(stale_id) = stale_id {
            remove_option(selectable.options, &stale_id);
        }
        sync_option(
            selectable.options,
            OptionState {
                tab_index: index(),
                value: value(),
                text_value: text_value.cloned(),
                id: option_id.clone(),
                disabled: disabled(),
            },
        );
        previous_id.set(Some(option_id));
    });

    use_effect_cleanup(move || {
        if let Some(option_id) = previous_id.peek().as_ref() {
            remove_option(selectable.options, option_id);
        }
    });

    use_focus_entry_disabled(selectable.focus, index, move || disabled.cloned());

    let selected =
        use_memo(move || selectable.selectable.cloned() && selectable.is_selected(&value()));

    use_context_provider(|| TagOptionCtx {
        value: value(),
        index,
    });

    let tabindex = use_memo(move || {
        if !(selectable.focus.roving_loop)() {
            return "0";
        }
        if selectable.focus.recent_focus_or_default() == index.cloned() {
            "0"
        } else {
            "-1"
        }
    });

    let onmounted = use_focus_controlled_item_disabled(index, move || disabled.cloned());
    let removable = ctx.allows_removing;

    if is_removed() {
        return rsx! {};
    }

    rsx! {
        div {
            role: "row",
            id: id(),
            tabindex,
            aria_rowindex: (index.cloned() as i32) + 1,
            aria_selected: (selectable.selectable)().then_some(selected()),
            aria_disabled: disabled(),
            "data-selected": selected(),
            "data-disabled": disabled(),
            onmounted,
            onfocus: move |_| selectable.focus.set_focus(Some(index.cloned())),
            onclick: move |_| {
                if !disabled() {
                    selectable.toggle_value(value());
                }
            },
            onkeydown: move |e| {
                tag_option_on_keydown(e, ctx, selectable, value(), disabled(), removable());
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

/// Remove button for the enclosing [`TagOption`]. Renders nothing when removal is disabled.
#[component]
pub fn TagRemoveButton(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut ctx: TagGroupCtx = use_context();
    let selectable = use_context::<TagSelectableContext>();
    let option: TagOptionCtx = use_context();
    if !ctx.is_removable() {
        return rsx! {};
    }

    let label = use_memo(move || {
        let text = selectable
            .options
            .read()
            .iter()
            .find(|o| o.tab_index == option.index.cloned())
            .map(|o| o.text_value.clone())
            .unwrap_or_default();
        format!("Remove item {text}")
    });

    rsx! {
        button {
            r#type: "button",
            tabindex: "-1",
            aria_label: "{label}",
            onclick: move |e| {
                e.stop_propagation();
                ctx.remove_value(selectable, option.value.clone());
            },
            ..attributes,
            {children}
        }
    }
}
