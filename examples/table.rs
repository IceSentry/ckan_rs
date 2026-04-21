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
    let mut rows = vec![];

    for i in 0..100 {
        let row = bsn! {
            // Need to use Children because tr is a bsn_list
            Node Children[{
                tr(bsn_list! {
                    Node Children[
                        {td(bsn_list!{
                            :label(format!("{i}"))
                        })},
                        {td(bsn_list!{
                            :label("test 1")
                        })},
                        {td(bsn_list!{
                            :label("test 2")
                        })}
                    ]
                })}
            ]
        };
        rows.push(row);
    }
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
                {rows}
            })
            // :tbody(bsn_list!{
            //     {tr(bsn_list!{
            //         {td(bsn_list!{
            //             :label("A")
            //         })},
            //         {td(bsn_list!{
            //             :label("B")
            //         })},
            //         {td(bsn_list!{
            //             :label("CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC")
            //         })},
            //     })},
            //     {tr(bsn_list!{
            //         {td(bsn_list!{
            //             :label("C")
            //         })},
            //         {td(bsn_list!{
            //             :label("D")
            //         })},
            //     })},
            // })
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

fn find_descendants(
    entity: Entity,
    children_q: &Query<&Children>,
    predicate: &impl Fn(Entity) -> bool,
) -> Vec<Entity> {
    let Ok(children) = children_q.get(entity) else {
        return vec![];
    };
    let mut result = vec![];
    for child in children.iter() {
        if predicate(child) {
            result.push(child);
        } else {
            result.extend(find_descendants(child, children_q, predicate));
        }
    }
    result
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
        info!("table spawned");

        // --- collect header TD widths in order ---
        let headers = find_descendants(table, &children, &|e| table_headers.get(e).is_ok());
        info!("headers found: {}", headers.len());

        let mut header_widths: Vec<f32> = Vec::new();
        for header in headers {
            let rows = find_descendants(header, &children, &|e| is_row.get(e).is_ok());
            info!("header rows found: {}", rows.len());
            for row in rows {
                let tds = find_descendants(row, &children, &|e| td_computed_node.get(e).is_ok());
                for td in tds {
                    let width = td_computed_node.get(td).unwrap().size().x;
                    info!("header td [{td}] width: {width}");
                    header_widths.push(width);
                }
            }
        }
        info!("header widths: {:?}", header_widths);

        // --- apply header widths to each body row's TDs in order ---
        let body = find_descendants(table, &children, &|e| is_body.get(e).is_ok());
        assert!(body.len() == 1, "Unexpectedly found multiple table body");

        let rows = find_descendants(body[0], &children, &|e| is_row.get(e).is_ok());
        for row in rows {
            let tds = find_descendants(row, &children, &|e| td_node.get(e).is_ok());
            for (col, td) in tds.into_iter().enumerate() {
                if let Some(&width) = header_widths.get(col) {
                    info!("setting body td [{td}] col {col} width to {width}");
                    td_node.get_mut(td).unwrap().width = Val::Px(width);
                }
            }
        }
    }
}
