use dioxus::prelude::*;
use dioxus_icons::lucide::X;
use dioxus_primitives::tag_group::{self, TagGroupCtx, TagGroupProps, TagListProps, TagProps};
use std::collections::HashSet;

#[css_module("/src/components/tag_group/style.css")]
struct Styles;

#[component]
pub fn TagGroup(props: TagGroupProps) -> Element {
    rsx! {
        tag_group::TagGroup {
            class: Styles::dx_tag_group,
            label: props.label,
            items: props.items,
            selection_mode: props.selection_mode,
            selected_tags: props.selected_tags,
            default_selected_tags: props.default_selected_tags,
            on_selection_change: props.on_selection_change,
            disabled_tags: props.disabled_tags,
            disabled: props.disabled,
            allows_empty_selection: props.allows_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            allows_removing: props.allows_removing,
            roving_loop: props.roving_loop,
            render_empty_state: props.render_empty_state,
            attributes: props.attributes,
            TagList { {props.children} }
        }
    }
}

#[component]
fn TagList(props: TagListProps) -> Element {
    let ctx: TagGroupCtx = use_context();
    let is_removable = ctx.is_removable();

    rsx! {
        tag_group::TagList {
            class: Styles::dx_tag_list,
            attributes: props.attributes,
            for item in tag_group::use_tag_list_items() {
                Tag {
                    key: "{item.key}",
                    index: item.index,
                    {item.children}
                    if is_removable {
                        RemoveButton { index: item.index }
                    }
                }
            }
            {props.children}
        }
    }
}

#[component]
pub fn Tag(props: TagProps) -> Element {
    rsx! {
        tag_group::Tag {
            class: Styles::dx_tag,
            index: props.index,
            disabled: props.disabled,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
fn RemoveButton(
    index: usize,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut ctx: TagGroupCtx = use_context();
    let tag_key = ctx.item_key(index);

    rsx! {
        button {
            class: Styles::dx_remove_button,
            r#type: "button",
            aria_label: format!("Remove item {tag_key}"),
            onclick: move |e| {
                e.stop_propagation();
                ctx.remove_tags(HashSet::from([tag_key.clone()]));
            },
            ..attributes,
            {children}
            X { size: "12px" }
        }
    }
}
