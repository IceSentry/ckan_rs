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
        .add_systems(Startup, setup)
        .add_systems(Update, send_scroll_events)
        .add_systems(PostUpdate, on_table_spawned.after(UiSystems::PostLayout))
        .add_observer(on_scroll_handler)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list![Camera2d, ui_root()]);
}

fn ui_root() -> impl Scene {
    let mut rows = vec![];

    for i in 0..100 {
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
            width: percent(100),
            height: percent(100),
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

#[derive(Component, Default, Clone)]
struct TableBody;
fn tbody(content: impl SceneList) -> impl Scene {
    bsn! {
        TableBody
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
            overflow: Overflow::scroll_y(),
        }
        Children[
            { content }
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

const SCROLL_SPEED: f32 = 42.;

fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= SCROLL_SPEED;
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

fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

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
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
