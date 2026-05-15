# Tag group

A Tag Group is a focusable group of tags (labels, categories, filters and similar items) with keyboard navigation, optional selection and removal.

## Structure

```rust
TagGroup {
    // Optional visible label for the group
    label,
    items,
    // The type of selection that is allowed in the group.
    selection_mode,
    // Controlled selection (keys match item `key` values)
    selected_tags,
    on_selection_change: move |tags| { /* ... */ },
    // Keys that cannot be selected or focused
    disabled_tags,
    // Show remove buttons; Delete/Backspace removes selected tags
    allows_removing,
    // Shown when `items` is empty
    render_empty_state,
}
```

## Item keys

Each entry in `items` should set a vnode `key`. Keys are used for selection, disabled state, and removal. If `key` is omitted, the item index is used as a string.
