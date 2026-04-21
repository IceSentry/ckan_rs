use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::display::label;
use bevy::feathers::theme::{ThemeBackgroundColor, UiTheme};
use bevy::feathers::{FeathersPlugins, tokens};
use bevy::prelude::*;
use bevy::ui::UiSystems;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, on_table_spawned.after(UiSystems::PostLayout))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list![Camera2d, ui_root()]);
}

fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        :table(bsn_list!{
            :thead(bsn_list!{
                {tr(bsn_list!{
                    {td(bsn_list!{
                        :label("AAA"),
                    })},
                    {td(bsn_list!{
                        :label("BBbbbbbbbbbbbbbbbbbBBBBBBBBB"),
                    })},
                    {td(bsn_list!{
                        :label("CCCCCCCCCCCC"),
                    })},
                })}
            }),
            :tbody(bsn_list!{
                {tr(bsn_list!{
                    {td(bsn_list!{
                        :label("A")
                    })},
                    {td(bsn_list!{
                        :label("B")
                    })},
                    {td(bsn_list!{
                        :label("CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC")
                    })},
                })},
                {tr(bsn_list!{
                    {td(bsn_list!{
                        :label("C")
                    })},
                    {td(bsn_list!{
                        :label("D")
                    })},
                })},
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
fn thead(content: impl SceneList) -> impl Scene {
    bsn! {
        TableHeader
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.),
        }
        Children[
            { content }
            :horizontal_serparator,
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
        }
        Children[
            { content }
        ]
    }
}

#[derive(Component, Default, Clone)]
struct TableRow;
fn tr(content: impl SceneList) -> impl SceneList {
    bsn_list! {
        (
            TableRow
            Node
            Children[{content}]
        ),
        :horizontal_serparator
    }
}

#[derive(Component, Default, Clone)]
struct TableData;
fn td(content: impl SceneList) -> impl SceneList {
    bsn_list! {
        (
            TableData
            Node {
                margin: UiRect::all(px(8.0))
                overflow: Overflow::clip(),
            }
            Children[{content}]
        ),
        :vertical_serparator
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
    tables: Query<&Children, Added<Table>>,
    children_q: Query<&Children>,
    header_q: Query<(), With<TableHeader>>,
    body_q: Query<(), With<TableBody>>,
    row_q: Query<(), With<TableRow>>,
    td_computed_q: Query<&ComputedNode, With<TableData>>,
    mut td_node_q: Query<&mut Node, With<TableData>>,
) {
    for table_children in &tables {
        info!("table spawned");

        // --- collect header TD widths in order ---
        let mut header_widths: Vec<f32> = Vec::new();
        for table_child in table_children.iter() {
            if header_q.get(table_child).is_err() {
                continue;
            }
            let Ok(header_children) = children_q.get(table_child) else {
                continue;
            };
            for header_child in header_children.iter() {
                if row_q.get(header_child).is_err() {
                    continue;
                }
                let Ok(row_children) = children_q.get(header_child) else {
                    continue;
                };
                for row_child in row_children.iter() {
                    if let Ok(computed) = td_computed_q.get(row_child) {
                        let width = computed.size().x;
                        info!("header td [{row_child}] width: {:?}", width);
                        header_widths.push(width);
                    }
                }
            }
        }
        info!("header widths: {:?}", header_widths);

        // --- apply header widths to each body row's TDs in order ---
        for table_child in table_children.iter() {
            if body_q.get(table_child).is_err() {
                continue;
            }
            let Ok(body_children) = children_q.get(table_child) else {
                continue;
            };
            for body_child in body_children.iter() {
                if row_q.get(body_child).is_err() {
                    continue;
                }
                let Ok(row_children) = children_q.get(body_child) else {
                    continue;
                };
                let mut col = 0;
                for row_child in row_children.iter() {
                    if let Ok(mut node) = td_node_q.get_mut(row_child) {
                        if let Some(&width) = header_widths.get(col) {
                            info!("setting body td [{row_child}] col {col} width to {width}");
                            node.width = Val::Px(width);
                        }
                        col += 1;
                    }
                }
            }
        }
    }
}
