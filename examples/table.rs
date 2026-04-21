use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::display::label;
use bevy::feathers::theme::{ThemeBackgroundColor, UiTheme};
use bevy::feathers::{FeathersPlugins, tokens};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup)
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
                            :label("hello"),
                        })},
                        {td(bsn_list!{
                            :label("world"),
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
