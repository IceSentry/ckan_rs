use argh::FromArgs;
use bevy::{
    ecs::schedule::SingleThreadedExecutor,
    feathers::{
        FeathersPlugins,
        dark_theme::create_dark_theme,
        display::{icon, label, label_dim},
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
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
        .add_observer(on_scroll_handler);

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
    app.edit_schedule(PostUpdate, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(Last, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });
    app.edit_schedule(Render, |s| {
        s.set_executor(SingleThreadedExecutor::new());
    });

    app.run();

    Ok(())
}

const LINE_HEIGHT: f32 = 22.;

fn setup(world: &mut World) -> Result {
    world.spawn_scene_list(bsn_list![Camera2d, layout_root()])?;
    Ok(())
}

#[derive(Component)]
struct GetList(Task<TaskResult>);

struct TaskResult {
    installed: Vec<ModuleRow>,
}

#[derive(Clone)]
struct ModuleRow {
    name: String,
    installed_version: String,
    latest_version: String,
}

fn startup_tasks(mut commands: Commands) {
    let pool = AsyncComputeTaskPool::get();
    let task = pool.spawn(async move {
        let _ = ckan::run_command(&["scan"]);
        let _ = ckan::run_command(&["update"]);

        let instance_path = ckan::default_instance_path().unwrap();
        let registry = ckan::get_registry(instance_path).unwrap();

        // TODO available
        let repo = ckan::get_repo(&registry).unwrap();
        //
        // for (module_id, module) in repo.available_modules {
        //     if let Some((version, _ckan_module)) = module.module_version.iter().last() {
        //         // println!("{module_id} ({version})");
        //     }
        // }

        let mut installed = vec![];
        for module in registry.installed_modules.values() {
            let module = &module.source_module;
            let repo_module = repo.available_modules.get(&module.identifier).unwrap();
            let (latest_version, _) = repo_module.module_version.iter().last().unwrap();
            installed.push(ModuleRow {
                name: module.name.clone(),
                installed_version: module.version.clone(),
                latest_version: latest_version.to_string(),
            });
        }
        // installed.sort_unstable();
        TaskResult { installed }
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

        let mut ui_root = commands.entity(*ui_root);
        ui_root.despawn_children();
        spawn_installed_table(&mut ui_root, &result.installed);
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

fn installed_row(row: ModuleRow) -> impl Scene {
    let bg_color = if row.installed_version != row.latest_version {
        Color::srgb(0.5, 0.0, 0.0)
    } else {
        Color::NONE
    };
    bsn! {
        Node {
            margin: UiRect::horizontal(px(10.0)),
            height: px(LINE_HEIGHT),
            width: percent(100),
        }
        BackgroundColor(bg_color)
        Children [
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(400.0),
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
                    width: px(150.0),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                }
                :label(row.installed_version.clone())
            ),
            :vertical_serparator,
            (
                Node {
                    margin: UiRect::horizontal(px(5.0)),
                    width: px(250.0),
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

fn spawn_installed_table(ui_root: &mut EntityCommands, installed: &[ModuleRow]) {
    for row in installed {
        let row = row.clone();
        ui_root.queue_spawn_related_scenes::<Children>(bsn_list! {(
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
            }
            Children [
                :installed_row(row),
                :horizontal_serparator
            ]
        )});
    }
}

#[derive(Component, Default, Clone, Copy)]
struct UiRoot;

#[derive(Component, Default, Clone, Copy)]
struct ScrollbarTrack;

#[derive(Component, Default, Clone, Copy)]
struct ScrollThumb;

fn layout_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            :ui_root,
            :scrollbar_track
        ]
    }
}

fn ui_root() -> impl Scene {
    bsn! {
        UiRoot
        Node {
            flex_grow: 1.,
            height: percent(100),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
        }
        Children[(
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
            }
            Children[(Text::new("Loading...") ThemedText)]
        )]
    }
}

fn scrollbar_track() -> impl Scene {
    bsn! {
        ScrollbarTrack
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
    ui_root: Single<(&ScrollPosition, &ComputedNode), With<UiRoot>>,
    track: Single<&ComputedNode, With<ScrollbarTrack>>,
    mut thumb: Single<&mut Node, With<ScrollThumb>>,
) {
    let (scroll_pos, computed) = *ui_root;
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
