use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::display::label;
use bevy::feathers::theme::{ThemeBackgroundColor, UiTheme};
use bevy::feathers::{FeathersPlugins, tokens};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::UiSystems;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .init_resource::<ThumbDrag>()
        .add_systems(Startup, setup)
        .add_systems(Update, (send_scroll_events, drag_thumb))
        .add_systems(
            PostUpdate,
            (on_table_spawned, update_scrollbar).after(UiSystems::PostLayout),
        )
        .add_observer(on_scroll_handler)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list![Camera2d, ui_root()]);
}

fn ui_root() -> impl Scene {
    let mut rows = vec![];

    for i in 0..420 {
        rows.push(bsn! {
            tr(bsn_list! {
                td(bsn!{
                    Node {
                        justify_content: JustifyContent::Center,
                        width: percent(100)
                    }
                    Children[ label(format!("{i}")) ]
                }),
                td(bsn!{ label("test 1") }),
                td(bsn!{ label("test 2") })
            })
        });
    }

    bsn! {
        Node {
            width: percent(80),
            height: percent(80),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        table(bsn_list!{
            thead(bsn!{
                tr(bsn_list!{
                    td(bsn!{ label("index") }),
                    td(bsn!{ label("Header ----------------------------------------------") }),
                    td(bsn!{ label("header") }),
                })
            }),
            tbody(bsn_list!{
                {rows}
            })
        })
    }
}

#[derive(Component, Default, Clone)]
struct Table;
fn table(content: impl SceneList) -> impl Scene {
    bsn! {
        Table
        Node {
            flex_direction: FlexDirection::Column,
        }
        Children[{ content }]
    }
}

#[derive(Component, Default, Clone)]
struct TableHeader;
fn thead(content: impl Scene) -> impl Scene {
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

fn ui_debug() -> impl Scene {
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
struct TableBody;
#[derive(Component, Default, Clone)]
struct TableBodyContent;
fn tbody(content: impl SceneList) -> impl Scene {
    bsn! {
        TableBody
        ui_debug()
        Node {
            flex_direction: FlexDirection::Row,
            overflow: Overflow::scroll_y(),
        }
        Children[
            (
                TableBodyContent
                ui_debug()
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
struct Scrollbar;

#[derive(Component, Default, Clone)]
struct ScrollbarThumb;

fn scrollbar_track() -> impl Scene {
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
struct TableRow;
fn tr(content: impl SceneList) -> impl Scene {
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
struct TableData;
fn td(content: impl Scene) -> impl Scene {
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

fn vertical_serparator() -> impl Scene {
    bsn!(
        Node {
            width: px(1),
            height: percent(100),
        }
        ThemeBackgroundColor(tokens::MENU_BORDER)
    )
}

fn horizontal_serparator() -> impl Scene {
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
    content_query: Query<&ComputedNode, With<TableBodyContent>>,
    rows: Query<(), With<TableRow>>,
) {
    let scroll_speed = content_query.single().map_or(1.0, |node| {
        let row_count = rows.iter().count();
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
struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

#[derive(Resource, Default)]
struct ThumbDrag {
    active: bool,
    last_mouse_y: f32,
}

fn drag_thumb(
    mut drag: ResMut<ThumbDrag>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    hover_map: Res<HoverMap>,
    windows: Query<&Window>,
    thumb_query: Query<(), With<ScrollbarThumb>>,
    scrollbar_query: Query<&ComputedNode, With<Scrollbar>>,
    mut content_query: Query<(&mut ScrollPosition, &ComputedNode), With<TableBodyContent>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    if mouse_buttons.just_released(MouseButton::Left) {
        drag.active = false;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        let thumb_hovered = hover_map
            .values()
            .flat_map(|m| m.keys().copied())
            .any(|e| thumb_query.contains(e));
        if thumb_hovered {
            drag.active = true;
            drag.last_mouse_y = window.cursor_position().map(|p| p.y).unwrap_or(0.);
        }
    }

    if !drag.active || !mouse_buttons.pressed(MouseButton::Left) {
        drag.active = false;
        return;
    }

    let Some(cursor_y) = window.cursor_position().map(|p| p.y) else {
        return;
    };

    let delta_y = cursor_y - drag.last_mouse_y;
    drag.last_mouse_y = cursor_y;

    if delta_y == 0. {
        return;
    }

    let Ok(track_node) = scrollbar_query.single() else {
        return;
    };
    let Ok((mut scroll_pos, content_node)) = content_query.single_mut() else {
        return;
    };

    let viewport_h = content_node.size().y;
    let content_h = content_node.content_size().y;
    let track_h = track_node.size().y;
    let scale = content_node.inverse_scale_factor();

    if content_h <= viewport_h || track_h <= 0. {
        return;
    }

    let thumb_h = (viewport_h / content_h * track_h).max(20.0);
    let thumb_travel = (track_h - thumb_h) * scale;
    let max_scroll = (content_h - viewport_h) * scale;

    if thumb_travel <= 0. {
        return;
    }

    let scroll_delta = delta_y * max_scroll / thumb_travel;
    scroll_pos.y = (scroll_pos.y + scroll_delta).clamp(0., max_scroll);
}

fn update_scrollbar(
    body_contents: Query<(&ScrollPosition, &ComputedNode), With<TableBodyContent>>,
    scrollbar: Query<&ComputedNode, With<Scrollbar>>,
    mut thumb: Query<&mut Node, With<ScrollbarThumb>>,
) {
    let Ok((scroll_pos, content_node)) = body_contents.single() else {
        return;
    };
    let Ok(track_node) = scrollbar.single() else {
        return;
    };
    let Ok(mut thumb_node) = thumb.single_mut() else {
        return;
    };

    let viewport_h = content_node.size().y;
    let content_h = content_node.content_size().y;
    let track_h = track_node.size().y;

    if content_h <= viewport_h || track_h <= 0. {
        thumb_node.height = Val::Percent(0.);
        return;
    }

    let scale = content_node.inverse_scale_factor();

    let thumb_h = (viewport_h / content_h * track_h).max(20.0);
    let max_scroll = (content_h - viewport_h) * scale;
    let scroll_ratio = (scroll_pos.y / max_scroll).clamp(0., 1.);
    let thumb_top = (track_h - thumb_h) * scroll_ratio;

    thumb_node.height = Val::Px(thumb_h * scale);
    thumb_node.top = Val::Px(thumb_top * scale);
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
