use bevy::{
    ecs::schedule::SingleThreadedExecutor,
    feathers::{
        FeathersPlugins,
        dark_theme::create_dark_theme,
        theme::{ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    prelude::*,
    render::Render,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
    winit::{EventLoopProxyWrapper, WinitSettings, WinitUserEvent},
};

mod ckan;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, (setup, startup_tasks))
        .add_systems(
            Update,
            (handle_tasks.run_if(any_with_component::<GetList>),),
        );

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

fn setup(world: &mut World) -> Result {
    world.spawn_scene_list(bsn_list![Camera2d, demo_root()])?;
    Ok(())
}

#[derive(Component)]
struct GetList(Task<TaskResult>);

struct TaskResult {
    installed: Vec<String>,
}

fn startup_tasks(mut commands: Commands) {
    let pool = AsyncComputeTaskPool::get();
    let task = pool.spawn(async move {
        let _ = ckan::run_command(&["scan"]);
        let _ = ckan::run_command(&["update"]);

        // let list = xshell::cmd!(sh, "./ckan.exe list --porcelain")
        //     .read()
        //     .expect("Failed to get list");
        //
        // let installed = list
        //     .lines()
        //     .map(|l| {
        //         let mut line_iter = l.split_whitespace();
        //         let status = line_iter.next().expect("status");
        //         let id = line_iter.next().expect("id").trim();
        //         let version = line_iter.next().expect("version").trim();
        //         ListEntry {
        //             status: ListEntryStatus::from_str(status),
        //             id: id.to_string(),
        //             version: version.to_string(),
        //         }
        //     })
        //     .collect::<Vec<_>>();

        let instance_path = ckan::default_instance_path().unwrap();
        let registry = ckan::get_registry(instance_path).unwrap();
        let repo = ckan::get_repo(&registry).unwrap();

        for (module_id, module) in repo.available_modules {
            if let Some((version, _ckan_module)) = module.module_version.iter().last() {
                // println!("{module_id} ({version})");
            }
        }

        let mut installed = vec![];
        for _ in 0..20 {
            installed.push("AAAAAAAAAAAAAAAAAAA".to_string());
        }
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
        for module in result.installed {
            ui_root.with_child((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                children![
                    (
                        Node {
                            margin: UiRect::horizontal(px(10.0)),
                            height: px(24),
                            width: percent(100),
                            flex_direction: FlexDirection::Row,
                            ..Default::default()
                        },
                        children![
                            (
                                Node {
                                    margin: UiRect::horizontal(px(5.0)),
                                    width: Val::Px(500.0),
                                    height: percent(100),
                                    overflow: Overflow::clip(),
                                    flex_direction: FlexDirection::Column,
                                    ..Default::default()
                                },
                                children![(Text::new(module), ThemedText)]
                            ),
                            // (
                            //     Node {
                            //         width: px(1),
                            //         height: percent(100),
                            //         ..Default::default()
                            //     },
                            //     ThemeBackgroundColor(tokens::MENU_BORDER)
                            // ),
                            // (
                            //     Node {
                            //         margin: UiRect::horizontal(px(5.0)),
                            //         width: Val::Px(140.0),
                            //         height: percent(100),
                            //         overflow: Overflow::clip(),
                            //         ..Default::default()
                            //     },
                            //     children![(Text::new(format!("{}", module.version)), ThemedText)]
                            // ),
                            // (
                            //     Node {
                            //         width: px(1),
                            //         height: percent(100),
                            //         ..Default::default()
                            //     },
                            //     ThemeBackgroundColor(tokens::MENU_BORDER)
                            // ),
                            // (
                            //     Node {
                            //         margin: UiRect::horizontal(px(5)),
                            //         // width: Val::Auto,
                            //         // height: px(16),
                            //         overflow: Overflow::clip(),
                            //         ..Default::default()
                            //     },
                            //     children![(Text::new(format!("{}", module.id)), ThemedText,)]
                            // )
                        ]
                    ),
                    (
                        Node {
                            height: px(1),
                            width: percent(100),
                            ..Default::default()
                        },
                        ThemeBackgroundColor(tokens::MENU_BORDER)
                    )
                ],
            ));
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
struct UiRoot;

fn demo_root() -> impl Scene {
    bsn! {
        UiRoot
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children[(
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::horizontal(px(10.0)),
            }
            Children[(
                Text::new("Loading...") ThemedText
            )]
        )]
    }
}
