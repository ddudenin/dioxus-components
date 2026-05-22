use dioxus::prelude::*;
use dioxus_icons::lucide::X;
use dioxus_primitives::tag_group::{
    self, TagGroupEmptyProps, TagGroupLabelProps, TagGroupMultiProps, TagGroupProps, TagListProps,
    TagOptionProps,
};

#[css_module("/src/components/tag_group/style.css")]
struct Styles;

#[component]
pub fn TagGroup(props: TagGroupProps<String>) -> Element {
    rsx! {
        tag_group::TagGroup {
            class: Styles::dx_tag_group,
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            selectable: props.selectable,
            allow_empty_selection: props.allow_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            roving_loop: props.roving_loop,
            attributes: props.attributes,
            TagList {
                {props.children}
            }
        }
    }
}

#[component]
pub fn TagGroupMulti(props: TagGroupMultiProps<String>) -> Element {
    rsx! {
        tag_group::TagGroupMulti {
            class: Styles::dx_tag_group,
            values: props.values,
            default_values: props.default_values,
            on_values_change: props.on_values_change,
            disabled: props.disabled,
            selectable: props.selectable,
            allow_empty_selection: props.allow_empty_selection,
            escape_clears_selection: props.escape_clears_selection,
            roving_loop: props.roving_loop,
            attributes: props.attributes,
            TagList {
                {props.children}
            }
        }
    }
}

#[component]
pub fn TagGroupLabel(props: TagGroupLabelProps) -> Element {
    rsx! {
        tag_group::TagGroupLabel {
            class: Styles::dx_tag_group_label,
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn TagGroupEmpty(props: TagGroupEmptyProps) -> Element {
    rsx! {
        tag_group::TagGroupEmpty {
            class: Styles::dx_tag_group_empty,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn TagList(props: TagListProps) -> Element {
    rsx! {
        tag_group::TagList {
            class: Styles::dx_tag_list,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn Tag(props: TagOptionProps<String>) -> Element {
    rsx! {
        tag_group::TagOption::<String> {
            class: Styles::dx_tag,
            value: props.value,
            text_value: props.text_value,
            disabled: props.disabled,
            is_removable: props.is_removable,
            id: props.id,
            index: props.index,
            attributes: props.attributes,
            {props.children}
            if props.is_removable.cloned() {
                RemoveButton {}
            }
        }
    }
}

#[component]
fn RemoveButton(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        tag_group::TagRemoveButton {
            class: Styles::dx_remove_button,
            attributes,
            {children}
            X { size: "12px" }
        }
    }
}
