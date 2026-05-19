# Tag group

A Tag Group is a focusable group of tags (labels, categories, filters and similar items) with keyboard navigation, optional selection and removal.

## Structure

Single selection with [`TagGroup`](component.rs):

```rust
TagGroup {
    label: "Labels",
    value: Some(value.into()),
    on_value_change: move |value| { /* ... */ },
    allows_removing: true,
    Tag { index: 0usize, value: "bug", "bug" }
    Tag { index: 1usize, value: "feature", disabled: true, "feature" }
}
```

Multiple selection with [`TagGroupMulti`](component.rs) — see the **multi** variant demo.
