//! List state for dynamic collections: items, selection, filtering, and list mutations.
//!
//! [`use_list_data`] keeps a list of items of type `T`, optional multi-select, and optional
//! filter text. Mutations (`append`, `remove`, `move_before`, …) update internal state; consumers
//! read the current collection from [`ListData::items`].
//!
//! # Item keys
//!
//! Each row needs a stable string key for selection, removal, and reordering. Keys come from the
//! `key` attribute on the [`Element`] returned by [`ListOptions::to_element`] (see [`element_key`]).
//! If no key is set, the item index is used as a string.
//!
//! ```rust
//! use dioxus::prelude::*;
//! use dioxus_primitives::list_data::{use_list_data, ListOptions};
//! use std::rc::Rc;
//!
//! #[derive(Clone, PartialEq)]
//! struct Row { id: u32, label: String }
//!
//! #[component]
//! fn Example() -> Element {
//!     let list = use_list_data(ListOptions {
//!         initial_items: vec![Row { id: 1, label: "News".into() }],
//!         to_element: Rc::new(|row| rsx! {
//!             span { key: "{row.id}", {row.label.clone()} }
//!         }),
//!         ..Default::default()
//!     });
//!
//!     rsx! {
//!         for (index, row) in (list.items)().into_iter().enumerate() {
//!             div { key: "{list.item_key(index)}", {row.label} }
//!         }
//!     }
//! }
//! ```
//!
//! # Selection
//!
//! Selection is controlled or uncontrolled via [`ListOptions::selected_keys`],
//! [`ListOptions::default_selected_keys`], and [`ListOptions::on_selected_keys_change`].
//! Use [`ListSelection::All`] or [`ListSelection::Keys`] for the current value.
//!
//! # Filtering
//!
//! When [`ListOptions::filter`] is set, [`ListData::items`] returns only items that match
//! [`ListData::filter_text`]. The full list remains in internal state; filtered rows are a view.
//!
//! # Wiring to UI
//!
//! Call [`use_list_data`] in the parent component, read [`ListData::items`] in `rsx!`, and use
//! callbacks such as [`ListData::remove`] and [`ListData::append`] to update the list.

use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

use crate::use_controlled;

/// Returns the `key` attribute value for `element`, or `index` when unset.
///
/// Used by [`use_list_data`] for stable row identity.
pub fn element_key(element: &Element, index: usize) -> String {
    element
        .as_ref()
        .ok()
        .and_then(|vnode| vnode.key.clone())
        .unwrap_or_else(|| index.to_string())
}

/// Stable string key for a row in the list.
pub type ListKey = String;

/// Converts a list item to an [`Element`] (must set a `key` attribute for stable identity).
pub type ToElementFn<T> = Rc<dyn Fn(&T) -> Element>;

/// Returns whether an item matches the current filter text.
pub type ListFilterFn<T> = Rc<dyn Fn(&T, &str) -> bool>;

/// Updates a list item from its previous value (used by [`UpdateValue::Map`]).
pub type ListItemUpdateFn<T> = Rc<dyn Fn(T) -> T>;

/// Either every item is selected or an explicit key set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListSelection {
    /// All items are selected.
    All,
    /// Selected keys only.
    Keys(HashSet<ListKey>),
}

#[derive(Debug, Clone, PartialEq)]
struct ListState<T: Clone + PartialEq> {
    items: Vec<T>,
}

/// Options for [`use_list_data`].
pub struct ListOptions<T: Clone + PartialEq + 'static> {
    /// Initial items in the list.
    pub initial_items: Vec<T>,
    /// Renders an item as an [`Element`] with a `key` attribute value (see [`element_key`]).
    pub to_element: ToElementFn<T>,
    /// The currently selected keys. `None` means uncontrolled.
    pub selected_keys: ReadSignal<Option<ListSelection>>,
    /// The default selected keys when uncontrolled.
    pub default_selected_keys: ListSelection,
    /// Called when the selection changes.
    pub on_selected_keys_change: Callback<ListSelection>,
    /// The current filter text. `None` means uncontrolled.
    pub filter_text: ReadSignal<Option<String>>,
    /// The default filter text when uncontrolled.
    pub default_filter_text: String,
    /// Called when the filter text changes.
    pub on_filter_text_change: Callback<String>,
    /// A function that returns whether a item matches the current filter text.
    pub filter: Option<ListFilterFn<T>>,
}

impl<T: Clone + PartialEq + 'static> Default for ListOptions<T> {
    fn default() -> Self {
        Self {
            initial_items: Vec::new(),
            to_element: Rc::new(|_| rsx! { span {} }),
            selected_keys: ReadSignal::new(Signal::new(None)),
            default_selected_keys: ListSelection::Keys(HashSet::new()),
            on_selected_keys_change: Callback::default(),
            filter_text: ReadSignal::new(Signal::new(None)),
            default_filter_text: String::new(),
            on_filter_text_change: Callback::default(),
            filter: None,
        }
    }
}

/// New value for [`ListData::update`].
#[derive(Clone)]
pub enum UpdateValue<T: Clone + PartialEq> {
    /// Replace the item.
    Replace(T),
    /// Update from the previous value.
    Map(ListItemUpdateFn<T>),
}

/// Handle returned by [`use_list_data`]. Clone to share the same list state across components.
pub struct ListData<T: Clone + PartialEq + 'static> {
    to_element: ToElementFn<T>,
    /// The items in the list (filtered when [`ListOptions::filter`] is set).
    pub items: Memo<Vec<T>>,
    /// The keys of the currently selected items in the list.
    pub selected_keys: Memo<ListSelection>,
    /// Sets the selected keys.
    pub set_selected_keys: Callback<ListSelection>,
    /// Adds the given keys to the current selected keys; pass [`ListSelection::All`] to select all keys.
    pub add_keys_to_selection: Callback<ListSelection>,
    /// Removes the given keys from the current selected keys; [`ListSelection::All`] clears to an empty key set.
    pub remove_keys_from_selection: Callback<ListSelection>,
    /// The current filter text.
    pub filter_text: Memo<String>,
    /// Sets the filter text (used with [`ListOptions::filter`]).
    pub set_filter_text: Callback<String>,
    /// Gets an item from the list by key.
    pub get_item: Callback<ListKey, Option<T>>,
    /// Inserts items into the list at the given `index` (clamped to end).
    pub insert: Callback<(usize, Vec<T>)>,
    /// Inserts items into the list before the item at the given `key` (or at start if the list is empty).
    pub insert_before: Callback<(ListKey, Vec<T>)>,
    /// Inserts items into the list after the item at the given `key`.
    pub insert_after: Callback<(ListKey, Vec<T>)>,
    /// Appends items to the list.
    pub append: Callback<Vec<T>>,
    /// Prepends items to the list.
    pub prepend: Callback<Vec<T>>,
    /// Removes items from the list by their keys.
    pub remove: Callback<HashSet<ListKey>>,
    /// Removes all items from the list that are currently in the set of selected items.
    pub remove_selected_items: Callback<()>,
    /// Moves an item within the list.
    pub r#move: Callback<(ListKey, usize)>,
    /// Moves one or more items before a given `key`.
    pub move_before: Callback<(ListKey, Vec<ListKey>)>,
    /// Moves one or more items after a given `key`.
    pub move_after: Callback<(ListKey, Vec<ListKey>)>,
    /// Updates an item in the list.
    pub update: Callback<(ListKey, UpdateValue<T>)>,
}

impl<T: Clone + PartialEq + 'static> ListData<T> {
    /// Returns the stable key for the item at `index` (vnode `key`, or `index` as string).
    pub fn item_key(&self, index: usize) -> String {
        (self.items)()
            .get(index)
            .map(|item| item_key(item, self.to_element.as_ref(), index))
            .unwrap_or_else(|| index.to_string())
    }
}

fn item_key<T>(item: &T, to_element: &dyn Fn(&T) -> Element, index: usize) -> String {
    element_key(&to_element(item), index)
}

fn all_keys<T>(items: &[T], to_element: &dyn Fn(&T) -> Element) -> HashSet<ListKey> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| item_key(item, to_element, index))
        .collect()
}

fn find_index<T>(items: &[T], to_element: &dyn Fn(&T) -> Element, key: &str) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .find(|(index, item)| item_key(*item, to_element, *index) == key)
        .map(|(index, _)| index)
}

/// Move indices `indices` (sorted ascending) to bucket starting at `to_index`
fn move_indices<T: Clone + PartialEq>(
    mut items: Vec<T>,
    mut indices: Vec<usize>,
    mut to_index: usize,
) -> Vec<T> {
    if indices.is_empty() {
        return items;
    }
    indices.sort_unstable();
    to_index -= indices.iter().filter(|&&i| i < to_index).count();

    let mut moves: Vec<(usize, usize)> = indices
        .into_iter()
        .enumerate()
        .map(|(k, from)| (from, to_index + k))
        .collect();

    for i in 0..moves.len() {
        let a = moves[i].0;
        for slot in moves.iter_mut().skip(i) {
            if slot.0 > a {
                slot.0 -= 1;
            }
        }
    }

    for i in 0..moves.len() {
        for j in (i + 1..moves.len()).rev() {
            if moves[j].0 < moves[i].1 {
                moves[i].1 += 1;
            } else {
                moves[j].0 += 1;
            }
        }
    }

    for (from, to) in moves {
        if from >= items.len() {
            continue;
        }
        let item = items.remove(from);
        let to = to.min(items.len());
        items.insert(to, item);
    }

    items
}

/// Creates list state for a dynamic collection.
///
/// Must be called at the top level of a component (like any Dioxus hook). Returns [`ListData`]
/// with reactive [`ListData::items`], selection, filter text, and callbacks to mutate the list.
pub fn use_list_data<T: Clone + PartialEq + 'static>(options: ListOptions<T>) -> ListData<T> {
    let to_element = options.to_element.clone();
    let filter = options.filter.clone();

    let state = use_signal(move || ListState {
        items: options.initial_items,
    });

    let (selected_keys, set_selected_keys) = use_controlled(
        options.selected_keys,
        options.default_selected_keys,
        options.on_selected_keys_change,
    );

    let (filter_text, set_filter_text) = use_controlled(
        options.filter_text,
        options.default_filter_text,
        options.on_filter_text_change,
    );

    let items = use_memo({
        let filter = filter.clone();
        move || {
            let s = state.read();
            match &filter {
                Some(f) => s
                    .items
                    .iter()
                    .filter(|item| f(item, filter_text().as_str()))
                    .cloned()
                    .collect(),
                None => s.items.clone(),
            }
        }
    });

    let add_keys_to_selection = use_callback({
        let to_element = to_element.clone();
        move |incoming: ListSelection| {
            let valid_keys = all_keys(&state.read().items, to_element.as_ref());
            match selected_keys() {
                ListSelection::All => (),
                ListSelection::Keys(cur) => match incoming {
                    ListSelection::All => set_selected_keys.call(ListSelection::All),
                    ListSelection::Keys(extra) => {
                        let mut next = cur;
                        for k in extra {
                            if valid_keys.contains(&k) {
                                next.insert(k);
                            }
                        }
                        set_selected_keys.call(ListSelection::Keys(next));
                    }
                },
            }
        }
    });

    let remove_keys_from_selection = use_callback({
        let to_element = to_element.clone();
        move |incoming: ListSelection| match incoming {
            ListSelection::All => {
                set_selected_keys.call(ListSelection::Keys(HashSet::new()));
            }
            ListSelection::Keys(to_remove) => {
                let items = state.read().items.clone();
                let mut all = match selected_keys() {
                    ListSelection::All => all_keys(&items, to_element.as_ref()),
                    ListSelection::Keys(cur) => cur,
                };
                for k in to_remove {
                    all.remove(&k);
                }
                set_selected_keys.call(ListSelection::Keys(all));
            }
        }
    });

    let get_item = use_callback({
        let to_element = to_element.clone();
        move |key: ListKey| {
            let s = state.read();
            let index = find_index(&s.items, to_element.as_ref(), &key)?;
            s.items.get(index).cloned()
        }
    });

    let insert = use_callback({
        let mut state = state;
        move |(index, values): (usize, Vec<T>)| {
            let mut w = state.write();
            let idx = index.min(w.items.len());
            w.items.splice(idx..idx, values);
        }
    });

    let insert_before = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |(key, values): (ListKey, Vec<T>)| {
            let mut w = state.write();
            let idx = match find_index(&w.items, to_element.as_ref(), &key) {
                Some(i) => i,
                None if w.items.is_empty() => 0,
                None => return,
            };
            w.items.splice(idx..idx, values);
        }
    });

    let insert_after = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |(key, values): (ListKey, Vec<T>)| {
            let mut w = state.write();
            let idx = match find_index(&w.items, to_element.as_ref(), &key) {
                Some(i) => i + 1,
                None if w.items.is_empty() => 0,
                None => return,
            };
            w.items.splice(idx..idx, values);
        }
    });

    let append = use_callback({
        let mut state = state;
        move |values: Vec<T>| {
            let mut w = state.write();
            let len = w.items.len();
            w.items.splice(len..len, values);
        }
    });

    let prepend = use_callback({
        let mut state = state;
        move |values: Vec<T>| {
            let mut w = state.write();
            w.items.splice(0..0, values);
        }
    });

    let remove = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |keys: HashSet<ListKey>| {
            let mut w = state.write();
            let keys_prune = keys.clone();
            let sel = selected_keys();
            let mut indices: Vec<usize> = w
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    keys.contains(&item_key(item, to_element.as_ref(), index))
                        .then_some(index)
                })
                .collect();
            indices.sort_unstable_by(|a, b| b.cmp(a));
            for index in indices {
                let _ = w.items.remove(index);
            }

            let next = if w.items.is_empty() {
                ListSelection::Keys(HashSet::new())
            } else {
                match sel {
                    ListSelection::All => ListSelection::All,
                    ListSelection::Keys(mut cur) => {
                        for k in keys_prune {
                            cur.remove(&k);
                        }
                        ListSelection::Keys(cur)
                    }
                }
            };
            set_selected_keys.call(next);
        }
    });

    let remove_selected_items = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |_| {
            let mut w = state.write();
            match selected_keys() {
                ListSelection::All => {
                    w.items.clear();
                }
                ListSelection::Keys(sel) => {
                    let mut indices: Vec<usize> = w
                        .items
                        .iter()
                        .enumerate()
                        .filter_map(|(index, item)| {
                            sel.contains(&item_key(item, to_element.as_ref(), index))
                                .then_some(index)
                        })
                        .collect();
                    indices.sort_unstable_by(|a, b| b.cmp(a));
                    for index in indices {
                        w.items.remove(index);
                    }
                }
            }
            set_selected_keys.call(ListSelection::Keys(HashSet::new()));
        }
    });

    let move_one = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |(key, to_index): (ListKey, usize)| {
            let mut w = state.write();
            let Some(from) = find_index(&w.items, to_element.as_ref(), &key) else {
                return;
            };
            let mut items = std::mem::take(&mut w.items);
            let item = items.remove(from);
            let at = to_index.min(items.len());
            items.insert(at, item);
            w.items = items;
        }
    });

    let move_before = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |(anchor_key, keys): (ListKey, Vec<ListKey>)| {
            let mut w = state.write();
            let Some(to_index) = find_index(&w.items, to_element.as_ref(), &anchor_key) else {
                return;
            };
            let mut indices: Vec<usize> = keys
                .iter()
                .filter_map(|k| find_index(&w.items, to_element.as_ref(), k))
                .collect();
            indices.sort_unstable();
            w.items = move_indices(std::mem::take(&mut w.items), indices, to_index);
        }
    });

    let move_after = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |(anchor_key, keys): (ListKey, Vec<ListKey>)| {
            let mut w = state.write();
            let Some(idx) = find_index(&w.items, to_element.as_ref(), &anchor_key) else {
                return;
            };
            let to_index = idx + 1;
            let mut indices: Vec<usize> = keys
                .iter()
                .filter_map(|k| find_index(&w.items, to_element.as_ref(), k))
                .collect();
            indices.sort_unstable();
            w.items = move_indices(std::mem::take(&mut w.items), indices, to_index);
        }
    });

    let update = use_callback({
        let mut state = state;
        let to_element = to_element.clone();
        move |(key, new_value): (ListKey, UpdateValue<T>)| {
            let mut w = state.write();
            let Some(i) = find_index(&w.items, to_element.as_ref(), &key) else {
                return;
            };
            let updated = match new_value {
                UpdateValue::Replace(v) => v,
                UpdateValue::Map(f) => f(w.items[i].clone()),
            };
            w.items[i] = updated;
        }
    });

    ListData {
        to_element,
        items,
        selected_keys,
        filter_text,
        set_selected_keys,
        add_keys_to_selection,
        remove_keys_from_selection,
        set_filter_text,
        get_item,
        insert,
        insert_before,
        insert_after,
        append,
        prepend,
        remove,
        remove_selected_items,
        r#move: move_one,
        move_before,
        move_after,
        update,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
    use std::rc::Rc;

    thread_local! {
        static DOM_TEST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    fn dom_test_fail(message: impl Into<String>) {
        DOM_TEST_ERROR.with(|slot| {
            if slot.borrow().is_none() {
                *slot.borrow_mut() = Some(message.into());
            }
        });
    }

    fn dom_test_finish() {
        DOM_TEST_ERROR.with(|slot| {
            if let Some(message) = slot.borrow_mut().take() {
                panic!("{message}");
            }
        });
    }

    /// Like [`assert!`] inside `run_dom!` — records failure for the outer `#[test]`.
    macro_rules! dom_assert {
        ($cond:expr $(,)?) => {
            if !$cond {
                dom_test_fail(format!(
                    "assertion failed: {}\n  at {}:{}",
                    stringify!($cond),
                    file!(),
                    line!(),
                ));
            }
        };
        ($cond:expr, $($arg:tt)+) => {
            if !$cond {
                dom_test_fail(format!(
                    "assertion failed: {}\n  at {}:{}",
                    format_args!($($arg)+),
                    file!(),
                    line!(),
                ));
            }
        };
    }

    /// Like [`assert_eq!`] inside `run_dom!` — records failure for the outer `#[test]`.
    macro_rules! dom_assert_eq {
        ($left:expr, $right:expr $(,)?) => {
            if $left != $right {
                dom_test_fail(format!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}\n  at {}:{}",
                    $left,
                    $right,
                    file!(),
                    line!(),
                ));
            }
        };
    }

    /// Run a `#[component]` that performs assertions during its first render.
    macro_rules! run_dom {
        ($component:path) => {{
            DOM_TEST_ERROR.with(|slot| *slot.borrow_mut() = None);
            let result = catch_unwind(AssertUnwindSafe(|| {
                let mut dom = VirtualDom::new($component);
                dom.rebuild_in_place();
            }));
            match result {
                Ok(()) => dom_test_finish(),
                Err(payload) => resume_unwind(payload),
            }
        }};
    }

    #[derive(Clone, PartialEq, Debug)]
    struct Row {
        id: String,
        n: i32,
    }

    fn row(id: &str, n: i32) -> Row {
        Row {
            id: id.to_string(),
            n,
        }
    }

    fn row_to_element(row: &Row) -> Element {
        rsx! {
            span { key: "{row.id}", "{row.id}" }
        }
    }

    fn item_keys(list: &ListData<Row>) -> Vec<String> {
        let items = (list.items)();
        (0..items.len()).map(|index| list.item_key(index)).collect()
    }

    fn test_options(
        initial_items: Vec<Row>,
        default_selected_keys: ListSelection,
        filter: Option<ListFilterFn<Row>>,
    ) -> ListOptions<Row> {
        ListOptions {
            initial_items,
            to_element: Rc::new(row_to_element),
            default_selected_keys,
            filter,
            ..Default::default()
        }
    }

    fn empty_keys() -> ListSelection {
        ListSelection::Keys(HashSet::new())
    }

    #[component]
    fn empty_data() -> Element {
        let list = use_list_data(test_options(vec![], empty_keys(), None));
        dom_assert!((list.items)().is_empty());
        dom_assert!(matches!(
            (list.selected_keys)(),
            ListSelection::Keys(k) if k.is_empty()
        ));
        dom_assert_eq!((list.filter_text)(), "");
        rsx! {
            div {}
        }
    }

    #[component]
    fn no_selection() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            ListSelection::Keys(HashSet::new()),
            None,
        ));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c"]);
        match (list.selected_keys)() {
            ListSelection::Keys(k) => dom_assert!(k.is_empty()),
            ListSelection::All => panic!("expected Keys"),
        }
        rsx! {
            div {}
        }
    }

    #[component]
    fn partial_selection() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            ListSelection::Keys(HashSet::from(["b".into()])),
            None,
        ));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c"]);
        match (list.selected_keys)() {
            ListSelection::Keys(k) => {
                dom_assert_eq!(k.len(), 1);
                dom_assert!(k.contains("b"));
            }
            ListSelection::All => panic!("expected Keys"),
        }
        rsx! {
            div {}
        }
    }

    #[component]
    fn selected_all() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            ListSelection::All,
            None,
        ));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c"]);
        match (list.selected_keys)() {
            ListSelection::Keys(_) => panic!("expected All"),
            ListSelection::All => (),
        }
        rsx! {
            div {}
        }
    }

    #[component]
    fn update_selection() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            ListSelection::Keys(HashSet::new()),
            None,
        ));

        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c"]);
        match (list.selected_keys)() {
            ListSelection::Keys(k) => dom_assert!(k.is_empty()),
            ListSelection::All => panic!("expected Keys"),
        }

        list.set_selected_keys.call(ListSelection::All);
        match (list.selected_keys)() {
            ListSelection::Keys(_) => panic!("expected All"),
            ListSelection::All => (),
        };

        list.remove_keys_from_selection
            .call(ListSelection::Keys(HashSet::from(["z".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => {
                dom_assert_eq!(k.len(), 3);
                dom_assert!(k.contains("b"));
            }
            ListSelection::All => panic!("expected Keys"),
        };

        list.remove_keys_from_selection
            .call(ListSelection::Keys(HashSet::from(["b".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => {
                dom_assert_eq!(k.len(), 2);
                dom_assert!(k.contains("a"));
                dom_assert!(k.contains("c"));
            }
            ListSelection::All => panic!("expected Keys"),
        };

        rsx! {
            div {}
        }
    }

    #[component]
    fn append_prepend_insert() -> Element {
        let list = use_list_data(test_options(vec![], empty_keys(), None));
        list.insert_before
            .call(("missing".into(), vec![row("m", 1)]));
        dom_assert_eq!(item_keys(&list), vec!["m"]);

        list.append.call(vec![row("a", 1)]);
        dom_assert_eq!(item_keys(&list), vec!["m", "a"]);

        list.prepend.call(vec![row("z", 9)]);
        dom_assert_eq!(item_keys(&list), vec!["z", "m", "a"]);

        list.insert.call((1, vec![row("x", 5)]));
        dom_assert_eq!(item_keys(&list), vec!["z", "x", "m", "a"]);

        list.insert.call((10, vec![row("v", 5)]));
        dom_assert_eq!(item_keys(&list), vec!["z", "x", "m", "a", "v"]);

        list.insert_before
            .call(("m".into(), vec![row("before_m", -1)]));
        dom_assert_eq!(item_keys(&list), vec!["z", "x", "before_m", "m", "a", "v"]);

        list.insert_before
            .call(("m2".into(), vec![row("failed", -1)]));
        dom_assert_eq!(item_keys(&list), vec!["z", "x", "before_m", "m", "a", "v"]);

        list.insert_after
            .call(("m".into(), vec![row("after_m", 2)]));
        dom_assert_eq!(
            item_keys(&list),
            vec!["z", "x", "before_m", "m", "after_m", "a", "v"]
        );

        list.insert_after
            .call(("m2".into(), vec![row("failed", -1)]));
        dom_assert_eq!(
            item_keys(&list),
            vec!["z", "x", "before_m", "m", "after_m", "a", "v"]
        );

        rsx! {
            div {}
        }
    }

    #[component]
    fn remove_and_get_item() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            empty_keys(),
            None,
        ));
        dom_assert_eq!(list.get_item.call("b".into()), Some(row("b", 2)));
        dom_assert_eq!(list.get_item.call("z".into()), None::<Row>);

        list.remove.call(HashSet::from(["b".into(), "c".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a"]);

        list.remove.call(HashSet::from(["z".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a"]);

        list.add_keys_to_selection.call(ListSelection::All);
        list.remove.call(HashSet::from(["a".into()]));
        dom_assert!((list.items)().is_empty());
        dom_assert!(matches!(
            (list.selected_keys)(),
            ListSelection::Keys(k) if k.is_empty()
        ));

        rsx! {
            div {}
        }
    }

    #[component]
    fn selection_set_add_remove() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2)],
            ListSelection::Keys(HashSet::from(["a".into()])),
            None,
        ));

        match (list.selected_keys)() {
            ListSelection::Keys(k) => {
                dom_assert!(k.len() == 1 && k.contains("a"));
            }
            _ => panic!("expected Keys"),
        }

        list.add_keys_to_selection
            .call(ListSelection::Keys(HashSet::from(["b".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => {
                dom_assert!(k.contains("a") && k.contains("b"));
            }
            _ => panic!("expected Keys"),
        }

        list.add_keys_to_selection
            .call(ListSelection::Keys(HashSet::from(["c".into(), "z".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => dom_assert_eq!(k.len(), 2),
            _ => panic!("expected Keys"),
        }

        list.add_keys_to_selection.call(ListSelection::All);
        dom_assert!(matches!((list.selected_keys)(), ListSelection::All));

        list.remove_keys_from_selection
            .call(ListSelection::Keys(HashSet::from(["a".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => dom_assert!(!k.contains("a")),
            _ => panic!("expected Keys after remove subset from All expansion"),
        }

        list.set_selected_keys
            .call(ListSelection::Keys(HashSet::from(["b".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => {
                dom_assert_eq!(k.len(), 1);
                dom_assert!(k.contains("b"));
            }
            _ => panic!("expected Keys"),
        }

        list.set_selected_keys
            .call(ListSelection::Keys(HashSet::from(["v".into()])));
        match (list.selected_keys)() {
            ListSelection::Keys(k) => dom_assert_eq!(k.len(), 1),
            _ => panic!("expected Keys"),
        }

        list.remove_keys_from_selection.call(ListSelection::All);
        match (list.selected_keys)() {
            ListSelection::Keys(k) => dom_assert!(k.is_empty()),
            _ => panic!("expected empty Keys"),
        }
        rsx! {
            div {}
        }
    }

    #[component]
    fn filter_and_filter_text() -> Element {
        let list = use_list_data(test_options(
            vec![row("apple", 1), row("banana", 2), row("apricot", 3)],
            empty_keys(),
            Some(Rc::new(|r: &Row, q: &str| {
                r.id.to_lowercase().contains(&q.to_lowercase())
            })),
        ));
        dom_assert_eq!((list.items)().len(), 3);

        list.set_filter_text.call("ap".into());
        dom_assert_eq!((list.filter_text)(), "ap");
        dom_assert_eq!((list.items)().len(), 2);

        list.set_filter_text.call(String::new());
        dom_assert_eq!((list.filter_text)(), "");
        dom_assert_eq!((list.items)().len(), 3);

        list.set_filter_text.call("42".into());
        dom_assert_eq!((list.filter_text)(), "42");
        dom_assert!((list.items)().is_empty());

        rsx! {
            div {}
        }
    }

    #[component]
    fn remove_selected_items() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            ListSelection::Keys(HashSet::from(["a".into(), "c".into()])),
            None,
        ));
        list.remove_selected_items.call(());
        dom_assert_eq!(item_keys(&list), vec!["b"]);
        dom_assert!(matches!(
            (list.selected_keys)(),
            ListSelection::Keys(k) if k.is_empty()
        ));

        let list = use_list_data(test_options(
            vec![row("x", 5), row("y", 2)],
            ListSelection::All,
            None,
        ));
        list.remove_selected_items.call(());
        dom_assert!((list.items)().is_empty());
        rsx! {
            div {}
        }
    }

    #[component]
    fn move_single() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2), row("c", 3)],
            empty_keys(),
            None,
        ));
        list.r#move.call(("c".into(), 0));
        dom_assert_eq!(item_keys(&list), vec!["c", "a", "b"]);

        list.r#move.call(("v".into(), 1));
        dom_assert_eq!(item_keys(&list), vec!["c", "a", "b"]);

        list.r#move.call(("c".into(), 7));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c"]);

        rsx! {
            div {}
        }
    }

    #[component]
    fn move_before() -> Element {
        let list = use_list_data(test_options(
            vec![
                row("a", 1),
                row("b", 2),
                row("c", 3),
                row("d", 4),
                row("e", 5),
            ],
            empty_keys(),
            None,
        ));

        list.move_before
            .call(("f".into(), vec!["b".into(), "e".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c", "d", "e"]);

        list.move_before
            .call(("d".into(), vec!["b".into(), "c".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c", "d", "e"]);

        list.move_before
            .call(("c".into(), vec!["b".into(), "e".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "e", "c", "d"]);

        list.move_before.call(("b".into(), vec!["c".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "c", "b", "e", "d"]);

        rsx! {
            div {}
        }
    }

    #[component]
    fn move_after() -> Element {
        let list = use_list_data(test_options(
            vec![
                row("a", 1),
                row("b", 2),
                row("c", 3),
                row("d", 4),
                row("e", 5),
            ],
            empty_keys(),
            None,
        ));

        list.move_after
            .call(("f".into(), vec!["b".into(), "e".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c", "d", "e"]);

        list.move_after
            .call(("с".into(), vec!["e".into(), "d".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "b", "c", "d", "e"]);

        list.move_after
            .call(("c".into(), vec!["e".into(), "b".into()]));
        dom_assert_eq!(item_keys(&list), vec!["a", "c", "b", "e", "d"]);

        list.move_after.call(("b".into(), vec!["a".into()]));
        dom_assert_eq!(item_keys(&list), vec!["c", "b", "a", "e", "d"]);

        rsx! {
            div {}
        }
    }

    #[component]
    fn update_replace_and_map() -> Element {
        let list = use_list_data(test_options(
            vec![row("a", 1), row("b", 2)],
            empty_keys(),
            None,
        ));
        list.update
            .call(("b".into(), UpdateValue::Replace(row("c", 99))));
        dom_assert_eq!((list.items)(), vec![row("a", 1), row("c", 99)]);

        list.update
            .call(("b".into(), UpdateValue::Replace(row("b", 99))));
        dom_assert_eq!((list.items)(), vec![row("a", 1), row("c", 99)]);

        list.update.call((
            "a".into(),
            UpdateValue::Map(Rc::new(|prev: Row| Row {
                n: prev.n + 10,
                ..prev
            })),
        ));
        dom_assert_eq!((list.items)(), vec![row("a", 11), row("c", 99)]);
        rsx! {
            div {}
        }
    }

    #[test]
    fn test_empty_data() {
        run_dom!(empty_data);
    }

    #[test]
    fn test_no_selection() {
        run_dom!(no_selection);
    }

    #[test]
    fn test_partial_selection() {
        run_dom!(partial_selection);
    }

    #[test]
    fn test_selected_all() {
        run_dom!(selected_all);
    }

    #[test]
    fn test_update_selection() {
        run_dom!(update_selection);
    }

    #[test]
    fn test_append_prepend_insert_before_after() {
        run_dom!(append_prepend_insert);
    }

    #[test]
    fn test_remove_and_get_item() {
        run_dom!(remove_and_get_item);
    }

    #[test]
    fn test_set_add_remove_selected_keys() {
        run_dom!(selection_set_add_remove);
    }

    #[test]
    fn test_filter() {
        run_dom!(filter_and_filter_text);
    }

    #[test]
    fn test_remove_selected_items() {
        run_dom!(remove_selected_items);
    }

    #[test]
    fn test_move_one_item() {
        run_dom!(move_single);
    }

    #[test]
    fn test_move_before() {
        run_dom!(move_before);
    }

    #[test]
    fn test_move_after() {
        run_dom!(move_after);
    }

    #[test]
    fn test_update_replace_and_map() {
        run_dom!(update_replace_and_map);
    }

    #[component]
    fn failing_dom_assert() -> Element {
        dom_assert_eq!(1, 2);
        rsx! {
            div {}
        }
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn run_dom_propagates_failed_assertion() {
        run_dom!(failing_dom_assert);
    }
}
