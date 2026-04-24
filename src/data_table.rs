use bevy::ecs::relationship::Relationship;
use bevy::feathers::{theme::ThemeBackgroundColor, tokens};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::UiSystems;

pub struct DataTablePlugin;
impl Plugin for DataTablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThumbDragActive>()
            .add_systems(Update, send_scroll_events)
            .add_systems(
                PostUpdate,
                (on_table_spawned, update_scrollbar).after(UiSystems::PostLayout),
            )
            .add_observer(on_scroll_handler)
            .add_observer(on_thumb_drag_start)
            .add_observer(on_thumb_drag)
            .add_observer(on_thumb_drag_end);
    }
}

#[derive(Component, Default, Clone)]
pub struct Table;
pub fn table(content: impl SceneList) -> impl Scene {
    bsn! {
        Table
        Node {
            flex_direction: FlexDirection::Column,
        }
        Children[{ content }]
    }
}

#[derive(Component, Default, Clone)]
pub struct TableHeader;
pub fn thead(content: impl Scene) -> impl Scene {
    bsn! {
        TableHeader
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
        }
        Children[
            content,
            horizontal_serparator(),
        ]
    }
}

pub fn ui_debug() -> impl Scene {
    bsn! {UiDebugOptions {
        enabled: true,
        outline_border_box: true,
        outline_padding_box: true,
        outline_content_box: true,
        outline_scrollbars: true,
        line_width: 0.25,
    }}
}

#[derive(Component, Default, Clone)]
pub struct TableBody;
#[derive(Component, Default, Clone)]
pub struct TableBodyContent;
pub fn tbody(content: impl SceneList) -> impl Scene {
    bsn! {
        TableBody
        Node {
            flex_direction: FlexDirection::Row,
            overflow: Overflow::scroll_y(),
        }
        Children[
            (
                TableBodyContent
                Node {
                    flex_direction: FlexDirection::Column,
                    width: percent(100),
                    height: percent(100),
                    overflow: Overflow::scroll_y(),
                }
                Children[{ content }]
            ),
            scrollbar_track()
        ]
    }
}

#[derive(Component, Default, Clone)]
pub struct Scrollbar;

#[derive(Component, Default, Clone)]
pub struct ScrollbarThumb;

pub fn scrollbar_track() -> impl Scene {
    bsn! {
        Scrollbar
        Node {
            width: px(12),
            height: percent(100),
        }
        ThemeBackgroundColor(tokens::SCROLLBAR_BG)
        Children[
            (
                ScrollbarThumb
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(0),
                    top: px(0),
                }
                ThemeBackgroundColor(tokens::SCROLLBAR_THUMB)
            )
        ]
    }
}

#[derive(Component, Default, Clone)]
pub struct TableRow;
pub fn tr(content: impl SceneList) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
        }
        Children[
            (
                TableRow Node Children[
                    {content}
                ]
            ),
            horizontal_serparator()
        ]
    }
}

#[derive(Component, Default, Clone)]
pub struct TableData;
pub fn td(content: impl Scene) -> impl Scene {
    bsn! {
        Node
        Children[
            (
                TableData
                Node {
                    margin: UiRect::all(px(8.0))
                    overflow: Overflow::clip(),
                }
                Children[content]
            ),
            vertical_serparator()
        ]
    }
}

pub fn vertical_serparator() -> impl Scene {
    bsn!(
        Node {
            width: px(1),
            height: percent(100),
        }
        ThemeBackgroundColor(tokens::MENU_BORDER)
    )
}

pub fn horizontal_serparator() -> impl Scene {
    bsn!(
        Node {
            height: px(1),
            width: percent(100),
        }
        ThemeBackgroundColor(tokens::MENU_BORDER)
    )
}

fn on_table_spawned(
    tables: Query<Entity, Added<Table>>,
    children: Query<&Children>,
    table_headers: Query<(), With<TableHeader>>,
    is_body: Query<(), With<TableBody>>,
    is_row: Query<(), With<TableRow>>,
    td_computed_node: Query<&ComputedNode, With<TableData>>,
    mut td_node: Query<&mut Node, With<TableData>>,
) {
    for table in &tables {
        // collect header TD widths in order
        let headers: Vec<Entity> = children
            .iter_descendants(table)
            .filter(|&e| table_headers.get(e).is_ok())
            .collect();

        let mut header_widths: Vec<f32> = Vec::new();
        for header in headers {
            let rows: Vec<Entity> = children
                .iter_descendants(header)
                .filter(|&e| is_row.get(e).is_ok())
                .collect();
            for row in rows {
                let tds: Vec<Entity> = children
                    .iter_descendants(row)
                    .filter(|&e| td_computed_node.get(e).is_ok())
                    .collect();
                for td in tds {
                    let width = td_computed_node.get(td).unwrap().size().x;
                    header_widths.push(width);
                }
            }
        }

        // apply header widths to each body row's TDs in order
        let body: Vec<Entity> = children
            .iter_descendants(table)
            .filter(|&e| is_body.get(e).is_ok())
            .collect();
        assert!(body.len() == 1, "Unexpectedly found multiple table body");

        let rows: Vec<Entity> = children
            .iter_descendants(body[0])
            .filter(|&e| is_row.get(e).is_ok())
            .collect();
        for row in rows {
            let tds: Vec<Entity> = children
                .iter_descendants(row)
                .filter(|&e| td_node.get(e).is_ok())
                .collect();
            for (col, td) in tds.into_iter().enumerate() {
                if let Some(&width) = header_widths.get(col) {
                    td_node.get_mut(td).unwrap().width = Val::Px(width);
                }
            }
        }
    }
}

fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut commands: Commands,
    content_query: Query<(Entity, &ComputedNode), With<TableBodyContent>>,
    rows: Query<(), With<TableRow>>,
    children_query: Query<&Children>,
    thumb_drag_active: Res<ThumbDragActive>,
) {
    if thumb_drag_active.0 {
        return;
    }
    let scroll_speed = content_query.iter().next().map_or(1.0, |(entity, node)| {
        let row_count = children_query
            .iter_descendants(entity)
            .filter(|&e| rows.contains(e))
            .count();
        if row_count > 0 {
            node.content_size().y * node.inverse_scale_factor() / row_count as f32
        } else {
            1.
        }
    });

    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= scroll_speed;
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

#[derive(Resource, Default)]
pub struct ThumbDragActive(bool);

fn on_thumb_drag_start(
    event: On<Pointer<DragStart>>,
    thumb_query: Query<(), With<ScrollbarThumb>>,
    mut active: ResMut<ThumbDragActive>,
) {
    if thumb_query.contains(event.event_target()) {
        active.0 = true;
    }
}

fn on_thumb_drag_end(
    event: On<Pointer<DragEnd>>,
    thumb_query: Query<(), With<ScrollbarThumb>>,
    mut active: ResMut<ThumbDragActive>,
) {
    if thumb_query.contains(event.event_target()) {
        active.0 = false;
    }
}

fn on_thumb_drag(
    event: On<Pointer<Drag>>,
    thumb_query: Query<(), With<ScrollbarThumb>>,
    thumb_parents: Query<&ChildOf, With<ScrollbarThumb>>,
    scrollbar_node: Query<(&ComputedNode, &ChildOf), With<Scrollbar>>,
    children: Query<&Children>,
    body_content_node: Query<&ComputedNode, With<TableBodyContent>>,
    mut commands: Commands,
) {
    if !thumb_query.contains(event.event_target()) {
        return;
    }

    let delta_y = event.delta.y;
    if delta_y == 0. {
        return;
    }

    let thumb = event.event_target();

    let Some((track_h, content_entity)) = (|| -> Option<(f32, Entity)> {
        let scrollbar_entity = thumb_parents.get(thumb).ok()?.get();
        let (track_node, scrollbar_parent) = scrollbar_node.get(scrollbar_entity).ok()?;
        let track_h = track_node.size().y;
        let content_entity = children
            .iter_descendants(scrollbar_parent.get())
            .find(|&e| body_content_node.contains(e))?;
        Some((track_h, content_entity))
    })() else {
        return;
    };

    let Ok(content_node) = body_content_node.get(content_entity) else {
        return;
    };

    let viewport_h = content_node.size().y;
    let content_h = content_node.content_size().y;
    let scale = content_node.inverse_scale_factor();

    if content_h <= viewport_h || track_h <= 0. {
        return;
    }

    let thumb_h = (viewport_h / content_h * track_h).max(20.0);
    let thumb_travel = (track_h - thumb_h) * scale;

    if thumb_travel <= 0. {
        return;
    }

    let max_scroll = (content_h - viewport_h) * scale;
    let scroll_delta = delta_y * max_scroll / thumb_travel;
    commands.trigger(Scroll {
        entity: content_entity,
        delta: Vec2::new(0., scroll_delta),
    });
}

fn update_scrollbar(
    body_contents: Query<(&ScrollPosition, &ComputedNode, &ChildOf), With<TableBodyContent>>,
    scrollbar_computed_nodes: Query<&ComputedNode, With<Scrollbar>>,
    mut thumb_nodes: Query<&mut Node, With<ScrollbarThumb>>,
    children_query: Query<&Children>,
) {
    for (scroll_pos, content_node, content_parent) in body_contents.iter() {
        // TODO consider caching this when the table spawns
        let Some(scrollbar_entity) = children_query
            .iter_descendants(content_parent.get())
            .find(|&e| scrollbar_computed_nodes.contains(e))
        else {
            continue;
        };

        let Some(thumb_entity) = children_query
            .iter_descendants(scrollbar_entity)
            .find(|&e| thumb_nodes.contains(e))
        else {
            continue;
        };

        let Ok(scrollbar_computed_node) = scrollbar_computed_nodes.get(scrollbar_entity) else {
            continue;
        };
        let Ok(mut thumb_node) = thumb_nodes.get_mut(thumb_entity) else {
            continue;
        };

        let viewport_h = content_node.size().y;
        let content_h = content_node.content_size().y;
        let track_h = scrollbar_computed_node.size().y;

        if content_h <= viewport_h || track_h <= 0. {
            thumb_node.height = Val::Percent(0.);
            continue;
        }

        let scale = content_node.inverse_scale_factor();

        let thumb_h = (viewport_h / content_h * track_h).max(20.0);
        let max_scroll = (content_h - viewport_h) * scale;
        let scroll_ratio = (scroll_pos.y / max_scroll).clamp(0., 1.);
        let thumb_top = (track_h - thumb_h) * scroll_ratio;

        thumb_node.height = Val::Px(thumb_h * scale);
        thumb_node.top = Val::Px(thumb_top * scale);
    }
}

// This only handles scrolling vertically
fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if delta.y == 0.0 {
        scroll.propagate(false);
    }
}
