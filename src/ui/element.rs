use bevy::prelude::*;

macro_rules! node_builder {
    ($attr: ident, $type: ty) => {
        pub const fn $attr(mut self, $attr: $type) -> Self {
            self.node.$attr = $attr.into();
            self
        }
    };
}

#[derive(Bundle, Default)]
pub struct Element {
    pub(crate) node: Node,
    pub(crate) background_color: BackgroundColor,
    pub(crate) border_color: BorderColor,
    pub(crate) global_z_index: GlobalZIndex,
    pub(crate) border_radius: BorderRadius,
}

impl Element {
    pub const fn new() -> Self {
        Self {
            node: Node::DEFAULT,
            background_color: BackgroundColor::DEFAULT,
            border_color: BorderColor::DEFAULT,
            global_z_index: GlobalZIndex(0),
            border_radius: BorderRadius::DEFAULT,
        }
    }
    node_builder!(width, Val);
    node_builder!(height, Val);
    node_builder!(row_gap, Val);
    node_builder!(flex_grow, f32);
    node_builder!(flex_direction, FlexDirection);
    node_builder!(justify_content, JustifyContent);
    node_builder!(align_items, AlignItems);
    node_builder!(border, UiRect);
    node_builder!(padding, UiRect);
    node_builder!(margin, UiRect);

    pub const fn background_color(mut self, background_color: impl const Into<Color>) -> Self {
        self.background_color = BackgroundColor(background_color.into());
        self
    }
    pub const fn border_color(mut self, border_color: impl const Into<BorderColor>) -> Self {
        self.border_color = border_color.into();
        self
    }
    pub const fn global_z_index(mut self, global_z_index: i32) -> Self {
        self.global_z_index = GlobalZIndex(global_z_index);
        self
    }
    pub const fn border_radius(mut self, border_radius: impl const Into<BorderRadius>) -> Self {
        self.border_radius = border_radius.into();
        self
    }
}
