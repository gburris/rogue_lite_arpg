use bevy::prelude::*;
use bon::{bon, builder};

#[allow(unused)]
pub struct Element {
    node: Node,
    background_color: BackgroundColor,
    border_color: BorderColor,
    global_z_index: GlobalZIndex,
    border_radius: BorderRadius,
}

#[bon]
impl Element {
    #[builder]
    pub fn new(
        #[builder(start_fn, into)] node: Node,
        #[builder(default, into)] background_color: BackgroundColor,
        #[builder(default, into)] border_color: BorderColor,
        #[builder(default, with = |z: i32| GlobalZIndex(z))] global_z_index: GlobalZIndex,
        #[builder(default, into)] border_radius: BorderRadius,
    ) -> impl Bundle {
        (
            node,
            background_color,
            border_color,
            global_z_index,
            border_radius,
        );
    }
}

#[builder(finish_fn = build)]
pub fn node(
    #[builder(default)] display: Display,
    #[builder(default)] box_sizing: BoxSizing,
    #[builder(default)] position_type: PositionType,
    #[builder(default)] left: Val,
    #[builder(default)] right: Val,
    #[builder(default)] top: Val,
    #[builder(default)] bottom: Val,
    #[builder(default)] flex_direction: FlexDirection,
    #[builder(default)] flex_wrap: FlexWrap,
    #[builder(default)] align_items: AlignItems,
    #[builder(default)] justify_items: JustifyItems,
    #[builder(default)] align_self: AlignSelf,
    #[builder(default)] justify_self: JustifySelf,
    #[builder(default)] align_content: AlignContent,
    #[builder(default)] justify_content: JustifyContent,
    #[builder(default)] margin: UiRect,
    #[builder(default)] padding: UiRect,
    #[builder(default)] border: UiRect,
    #[builder(default)] flex_grow: f32,
    #[builder(default)] flex_shrink: f32,
    #[builder(default)] flex_basis: Val,
    #[builder(default)] width: Val,
    #[builder(default)] height: Val,
    #[builder(default)] min_width: Val,
    #[builder(default)] min_height: Val,
    #[builder(default)] max_width: Val,
    #[builder(default)] max_height: Val,
    aspect_ratio: Option<f32>,
    #[builder(default)] overflow: Overflow,
    #[builder(default)] overflow_clip_margin: OverflowClipMargin,
    #[builder(default)] scrollbar_width: f32,
    #[builder(default)] row_gap: Val,
    #[builder(default)] column_gap: Val,
    #[builder(default)] grid_auto_flow: GridAutoFlow,
    #[builder(default)] grid_template_rows: Vec<RepeatedGridTrack>,
    #[builder(default)] grid_template_columns: Vec<RepeatedGridTrack>,
    #[builder(default)] grid_auto_rows: Vec<GridTrack>,
    #[builder(default)] grid_auto_columns: Vec<GridTrack>,
    #[builder(default)] grid_column: GridPlacement,
    #[builder(default)] grid_row: GridPlacement,
) -> Node {
    Node {
        display,
        box_sizing,
        position_type,
        left,
        right,
        top,
        bottom,
        flex_direction,
        flex_wrap,
        align_items,
        justify_items,
        align_self,
        justify_self,
        align_content,
        justify_content,
        margin,
        padding,
        border,
        flex_grow,
        flex_shrink,
        flex_basis,
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
        aspect_ratio,
        overflow,
        overflow_clip_margin,
        scrollbar_width,
        row_gap,
        column_gap,
        grid_auto_flow,
        grid_template_rows,
        grid_template_columns,
        grid_auto_rows,
        grid_auto_columns,
        grid_column,
        grid_row,
    }
}

impl Into<Node> for NodeBuilder {
    fn into(self) -> Node {
        self.build()
    }
}
