use argh::FromArgs;
use bevy::{
    ecs::schedule::SingleThreadedExecutor,
    feathers::{
        FeathersPlugins,
        dark_theme::create_dark_theme,
        display::label,
        theme::{ThemeBackgroundColor, ThemeFontColor, ThemedText, UiTheme},
        tokens,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
    render::Render,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
    winit::{EventLoopProxyWrapper, WinitSettings, WinitUserEvent},
};

mod ckan;

/// CKAN mod manager
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    command: Option<Command>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    List(ListCommand),
}

/// Print the list of installed mods and exit
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
struct ListCommand {}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    if matches!(args.command, Some(Command::List(_))) {
        let _ = ckan::run_command(&["scan"]);
        let instance_path = ckan::default_instance_path()?;
        let registry = ckan::get_registry(instance_path)?;
        let mut installed: Vec<String> = registry
            .installed_modules
            .values()
            .map(|m| m.source_module.name.clone())
            .collect();
        installed.sort_unstable();
        for name in installed {
            println!("{name}");
        }
        return Ok(());
    }

    let mut app = App::new();
    app.add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, (setup, startup_tasks))
        .add_systems(
            Update,
            (handle_tasks.run_if(any_with_component::<GetList>),),
        )
        .add_systems(Update, send_scroll_events)
        .add_systems(Update, update_scroll_indicator)
        .add_observer(on_scroll_handler)
        .add_observer(on_thumb_drag)
        .add_observer(spawn_rows);

    // Set all schedules to single threaded to reduce cpu usage
    app.edit_schedule(First, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(PreUpdate, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(Update, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(SpawnScene, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(PostUpdate, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(Last, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });

    app.edit_schedule(FixedFirst, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(FixedPreUpdate, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(FixedUpdate, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(FixedPostUpdate, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(FixedLast, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });

    app.edit_schedule(Render, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(ExtractSchedule, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });

    app.run();

    Ok(())
}

const LINE_HEIGHT: f32 = 22.;

fn setup(world: &mut World) -> Result {
    world.spawn_scene_list(bsn_list![Camera2d, ui_root()])?;
    Ok(())
}

#[derive(Component)]
struct GetList(Task<TaskResult>);

struct TaskResult {
    list: Vec<ModuleRow>,
}

#[derive(Clone)]
struct ModuleRow {
    name: String,
    installed_version: Option<String>,
    latest_version: String,
}

fn startup_tasks(mut commands: Commands) {
    let pool = AsyncComputeTaskPool::get();
    let task = pool.spawn(async move {
        // info!("ckan scan");
        // let _ = ckan::run_command(&["scan"]);
        // info!("ckan update");
        // let _ = ckan::run_command(&["update"]);

        let instance_path = ckan::default_instance_path().unwrap();
        info!("Getting ckan registry");
        let registry = ckan::get_registry(instance_path).unwrap();

        info!("Getting repo from registry");
        let repo = ckan::get_repo(&registry).unwrap();

        info!("Populating installed list");
        let mut list = vec![];
        for (module_id, versions) in repo.available_modules {
            let (latest_version, module_metadata) = versions.module_version.iter().last().unwrap();
            let installed_version = registry
                .installed_modules
                .get(&module_id)
                .map(|installed_module| installed_module.source_module.version.clone());
            list.push(ModuleRow {
                name: module_metadata.name.clone(),
                installed_version,
                latest_version: latest_version.clone(),
            })
        }

        list.sort_unstable_by(|a, b| {
            b.installed_version
                .is_some()
                .cmp(&a.installed_version.is_some())
                .then_with(|| a.name.cmp(&b.name))
        });
        info!("Tasks done");
        // installed.sort_unstable();
        TaskResult { list }
    });
    commands.spawn(GetList(task));
}

fn handle_tasks(
    mut commands: Commands,
    mut transform_tasks: Query<(Entity, &mut GetList)>,
    ui_root: Single<Entity, With<UiRoot>>,
    event_loop_proxy: Res<EventLoopProxyWrapper>,
) {
    // Keep the app awake until the task is complete
    let _ = event_loop_proxy.send_event(WinitUserEvent::WakeUp);

    for (entity, mut task) in &mut transform_tasks {
        let Some(result) = check_ready(&mut task.0) else {
            continue;
        };
        commands.entity(entity).remove::<GetList>();

        // Despawn everything
        let mut ui_root = commands.entity(*ui_root);
        ui_root.despawn_children();

        ui_root.queue_spawn_related_scenes::<Children>(spawn_table(result.list.clone()));
    }
}

fn ui_root() -> impl Scene {
    bsn! {
        UiRoot
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children[(
            Node {
                flex_grow: 1.,
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            Children[(Text::new("Loading...") ThemedText)]
        )]
    }
}

fn spawn_table(rows: Vec<ModuleRow>) -> impl SceneList {
    bsn_list! {
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.,
        }
        Children[
            // Header
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                }
                Children[
                    (
                        Node {
                            height: Val::Px(LINE_HEIGHT),
                            width: Val::Percent(100.),
                        }
                        :table_header
                    )
                    :horizontal_serparator,
                ]
            ),
            // Row + Scroll
            (
                Node {
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Row,
                    min_height: Val::Px(0.),
                }
                Children[
                    (
                        TableRows::new(rows.clone())
                        Node {
                            flex_grow: 1.,
                            height: Val::Percent(100.),
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                        }
                    ),
                    :table_scrollbar
                ]
            )
        ]
    }
}

const COL_SIZE: [f32; 4] = [120.0, 400.0, 220.0, 220.0];

fn table_header() -> impl Scene {
    bsn! {
        Children[
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[0]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                :label_bold("Installed")
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[1]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label_bold("Name")
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[2]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label_bold("Installed Version")
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[3]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label_bold("Latest Version")
            ),
            :vertical_serparator
        ]
    }
}

pub fn label_bold(text: impl Into<String>) -> impl Scene {
    let text = Text::new(text.into());
    bsn! {
        Node
        ThemeFontColor(tokens::TEXT_MAIN)
        // InheritableFont {
        //     font: fonts::REGULAR,
        //     font_size: size::MEDIUM_FONT,
        //     weight: FontWeight::EXTRA_BOLD,
        // }
        Children [
            template_value(text)
            ThemedText
        ]
    }
}

fn spawn_rows(event: On<Add, TableRows>, mut commands: Commands, q: Query<&TableRows>) {
    let mut content = commands.entity(event.event_target());
    for row in &q.get(event.event_target()).unwrap().0 {
        let row = row.clone();
        content.queue_spawn_related_scenes::<Children>(bsn_list! {(
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
            }
            Children [
                :module_row(row),
                :horizontal_serparator
            ]
        )});
    }
}

fn module_row(row: ModuleRow) -> impl Scene {
    let bg_color = if row
        .installed_version
        .as_ref()
        .is_some_and(|v| *v != row.latest_version)
    {
        Color::srgb(0.5, 0.0, 0.0)
    } else {
        Color::NONE
    };
    let installed_label = if row.installed_version.is_some() {
        "x"
    } else {
        ""
    };
    let installed_version = if let Some(v) = row.installed_version {
        v.clone()
    } else {
        "-".to_string().clone()
    };

    bsn! {
        Node {
            height: px(LINE_HEIGHT),
            width: percent(100),
        }
        BackgroundColor(bg_color)
        Children [
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[0]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                :label(installed_label)
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[1]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label(row.name.clone())
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[2]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label(installed_version)
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(COL_SIZE[3]),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label(row.latest_version.clone())
            ),
            :vertical_serparator
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

#[derive(Component, Default, Clone, Copy)]
struct UiRoot;

#[derive(Component, Default, Clone)]
struct TableRows(Vec<ModuleRow>);

impl TableRows {
    fn new(list: Vec<ModuleRow>) -> Self {
        Self(list)
    }
}

#[derive(Component, Default, Clone, Copy)]
struct TableScrollbar;

#[derive(Component, Default, Clone, Copy)]
struct ScrollThumb;

fn table_scrollbar() -> impl Scene {
    bsn! {
        TableScrollbar
        Node {
            width: px(8),
            height: percent(100),
        }
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1))
        Children[(
            ScrollThumb
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                width: percent(100),
                height: px(0),
            }
            BackgroundColor(Color::srgb(0.5, 0.5, 0.5))
        )]
    }
}

fn update_scroll_indicator(
    content_pane: Option<Single<(&ScrollPosition, &ComputedNode), With<TableRows>>>,
    track: Option<Single<&ComputedNode, With<TableScrollbar>>>,
    thumb: Option<Single<&mut Node, With<ScrollThumb>>>,
) {
    let (Some(content_pane), Some(track), Some(mut thumb)) = (content_pane, track, thumb) else {
        return;
    };
    let (scroll_pos, computed) = *content_pane;
    let scale = computed.inverse_scale_factor();
    let viewport_h = computed.size().y * scale;
    let content_h = computed.content_size().y * scale;

    if content_h <= viewport_h {
        thumb.height = Val::Px(0.);
        return;
    }

    let track_h = track.size().y * track.inverse_scale_factor();
    let thumb_h = (viewport_h / content_h * track_h).max(20.);
    let max_scroll = content_h - viewport_h;
    let thumb_top = scroll_pos.y / max_scroll * (track_h - thumb_h);

    thumb.height = Val::Px(thumb_h);
    thumb.top = Val::Px(thumb_top);
}

fn on_thumb_drag(
    drag: On<Pointer<Drag>>,
    thumb_query: Query<(), With<ScrollThumb>>,
    content_pane: Option<Single<(&mut ScrollPosition, &ComputedNode), With<TableRows>>>,
    track: Option<Single<&ComputedNode, With<TableScrollbar>>>,
) {
    if thumb_query.get(drag.event_target()).is_err() {
        return;
    }
    let (Some(content_pane), Some(track)) = (content_pane, track) else {
        return;
    };
    let (mut scroll_pos, computed) = content_pane.into_inner();
    let scale = computed.inverse_scale_factor();
    let viewport_h = computed.size().y * scale;
    let content_h = computed.content_size().y * scale;
    if content_h <= viewport_h {
        return;
    }
    let track_h = track.size().y * track.inverse_scale_factor();
    let thumb_h = (viewport_h / content_h * track_h).max(20.);
    let max_scroll = content_h - viewport_h;
    let scroll_range = track_h - thumb_h;
    if scroll_range <= 0. {
        return;
    }
    scroll_pos.y = (scroll_pos.y + drag.delta.y / scroll_range * max_scroll).clamp(0., max_scroll);
}

fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT * 2.0;
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
