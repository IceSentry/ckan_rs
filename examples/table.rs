use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::display::label;
use bevy::feathers::theme::{ThemeBackgroundColor, UiTheme};
use bevy::feathers::{FeathersPlugins, tokens};
use bevy::prelude::*;

use ckan_rs::data_table::{DataTablePlugin, table, tbody, td, thead, tr};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins, DataTablePlugin))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_scene_list(bsn_list![Camera2d, ui_root()]);
}

fn ui_root() -> impl Scene {
    let mut rows = vec![];
    let mut rows_2 = vec![];

    for i in 0..42 {
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
        rows_2.push(bsn! {
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
        Children[
            (
                Node {
                    width: percent(50),
                }
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
            ),
            (
                Node {
                    width: percent(50),
                }
                table(bsn_list!{
                    thead(bsn!{
                        tr(bsn_list!{
                            td(bsn!{ label("index") }),
                            td(bsn!{ label("Header ----------------------------------------------") }),
                            td(bsn!{ label("header") }),
                        })
                    }),
                    tbody(bsn_list!{
                        {rows_2}
                    })
                })
            )
        ]
    }
}
