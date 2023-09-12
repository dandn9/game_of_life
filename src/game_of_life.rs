use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::{PresentMode, WindowResolution},
};

const CELL_SIZE: u32 = 5;
const TIME_STEP_SECS: f64 = 0.1;

////////////////////////////////////////////////////////////////////////
/// COMPONENTS
///////////////////////////////////////////////////////////////////////

#[derive(Component, Debug, Copy, Clone)]
enum State {
    ALIVE,
    DEAD,
}
#[derive(Component, Debug)]
struct Cell {
    state: State,
    next_state: Option<State>,
}

#[derive(Component)]
struct FPSCounter;

////////////////////////////////////////////////////////////////////////
/// RESOURCES
///////////////////////////////////////////////////////////////////////
#[derive(Resource, Debug)]
struct EntityMap {
    v: Vec<Vec<Entity>>,
}

#[derive(Resource)]
struct StateMaterials {
    alive_material: Handle<ColorMaterial>,
    dead_material: Handle<ColorMaterial>,
}

#[derive(Resource, Debug, Clone)]
struct LastUpdate(f64);

////////////////////////////////////////////////////////////////////////

pub fn main() {
    // console_log::init_with_level(Level::DEBUG);
    info!("TEST INFOO");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::default(),
                present_mode: PresentMode::AutoNoVsync,

                canvas: Some("#my-canvas".to_string()),

                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, seed) // commands need to be flushed
        .add_systems(
            Update,
            (
                (process_cells, update_colors.after(process_cells))
                    .run_if(should_next_tick(TIME_STEP_SECS)),
                (update_ui).run_if(should_update_counter(1.)), // Update the fps counter every 1 second
            ),
        )
        .run();
}

////////////////////////////////////////////////////////////////////////
/// RUN CONDITIONS
////////////////////////////////////////////////////////////////////////

fn should_update_counter(interval: f64) -> impl FnMut(Local<f64>, Res<Time>) -> bool {
    move |mut prev_interval: Local<f64>, time: Res<Time>| {
        if *prev_interval >= interval {
            *prev_interval = 0.;
            true
        } else {
            *prev_interval += time.delta_seconds_f64();
            false
        }
    }
}

// Decides if the `evolution` systems run
fn should_next_tick(t: f64) -> impl FnMut(Local<f64>, Res<Time>) -> bool {
    move |mut previous_tick: Local<f64>, time: Res<Time>| {
        // Tick the timer
        if time.elapsed_seconds_f64() - (*previous_tick) >= t {
            *previous_tick = time.elapsed_seconds_f64();
            true
        } else {
            false
        }
    }
}
////////////////////////////////////////////////////////////////////////
/// SYSTEMS
///////////////////////////////////////////////////////////////////////

fn update_ui(time: Res<Time>, mut counter: Query<&mut Text, With<FPSCounter>>) {
    let delta_time = time.delta_seconds_f64();
    let fps = (1. / delta_time) as i32;

    let mut text = counter.single_mut();
    if let Some(section) = text.sections.first_mut() {
        section.value = fps.to_string();
    }
}

// Logic to determine whether a cell should be alive in the next tick or not
fn is_cell_alive(cell: &Cell, neighbours: [Option<&Cell>; 8]) -> bool {
    // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules

    let mut neighbours_alive = 0;

    for neighbour in neighbours {
        match neighbour {
            Some(neighbour_cell) => match (*neighbour_cell).state {
                State::ALIVE => neighbours_alive += 1,
                State::DEAD => {}
            },
            None => {}
        }
    }

    match (*cell).state {
        State::ALIVE => {
            if neighbours_alive < 2 {
                return false;
            }
            if neighbours_alive == 2 || neighbours_alive == 3 {
                return true;
            } else {
                return false;
            };
        }
        State::DEAD => {
            if neighbours_alive == 3 {
                return true;
            } else {
                return false;
            }
        }
    }
}

// Updates the next_state of the cells and after all the cells have been updated, state=next_state
fn process_cells(mut cells: Query<&mut Cell>, entities_map: ResMut<EntityMap>) {
    for row_index in 0..entities_map.v.len() {
        let row = entities_map.v.get(row_index).unwrap();
        for col_index in 0..row.len() {
            let entity = row.get(col_index).unwrap().clone();

            // Get the neighbours
            let prev_row = entities_map.v.get((row_index as i32 - 1) as usize);
            let next_row = entities_map.v.get(row_index + 1);

            let neighbors0 = prev_row.and_then(|f| {
                f.get((col_index as i32 - 1) as usize)
                    .and_then(|f| cells.get(f.clone()).ok())
            });
            let neighbors1 =
                prev_row.and_then(|f| f.get(col_index + 0).and_then(|f| cells.get(f.clone()).ok()));
            let neighbors2 =
                prev_row.and_then(|f| f.get(col_index + 1).and_then(|f| cells.get(f.clone()).ok()));

            let neighbors3 = row
                .get((col_index as i32 - 1) as usize)
                .and_then(|f| cells.get(f.clone()).ok());
            let neighbors5 = row
                .get(col_index + 1)
                .and_then(|f| cells.get(f.clone()).ok());

            let neighbors6 = next_row.and_then(|f| {
                f.get((col_index as i32 - 1) as usize)
                    .and_then(|f| cells.get(f.clone()).ok())
            });
            let neighbors7 =
                next_row.and_then(|f| f.get(col_index + 0).and_then(|f| cells.get(f.clone()).ok()));
            let neighbors8 =
                next_row.and_then(|f| f.get(col_index + 1).and_then(|f| cells.get(f.clone()).ok()));

            let cell = cells.get(entity).ok().unwrap();
            let is_alive = is_cell_alive(
                &cell,
                [
                    neighbors0, neighbors1, neighbors2, neighbors3, neighbors5, neighbors6,
                    neighbors7, neighbors8,
                ],
            );

            // mutate the cell after all the immutable references  ~_~
            let mut mutable_cell = cells.get_mut(entity).ok().unwrap();
            if is_alive {
                mutable_cell.next_state = Some(State::ALIVE)
            } else {
                mutable_cell.next_state = Some(State::DEAD)
            }
        }
    }

    // re loop all the cells to update the next_state
    for mut cell in cells.iter_mut() {
        cell.state = cell.next_state.unwrap();
    }
}

// Seeds the state of the board (for now just a simple 50%)
fn seed(mut query: Query<&mut Cell>) {
    for mut cell in query.iter_mut() {
        let rand = rand::random::<f32>();

        if rand >= 0.8 {
            cell.state = State::ALIVE
        }
    }
}

fn update_colors(mut query: Query<(&Cell, &mut Sprite)>, state_materials: Res<StateMaterials>) {
    for (cell, mut sprite) in query.iter_mut() {
        match cell.state {
            State::ALIVE => {
                sprite.color = Color::WHITE;
                // *material = state_materials.alive_material.clone();
            }
            State::DEAD => {
                sprite.color = Color::BLACK;
            }
        }
    }
}

// Creates the entities and resources
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    win_q: Query<&Window>,
) {
    let win = win_q.single();
    let rows = (win.width() / CELL_SIZE as f32).floor() as u32;
    let columns = (win.height() / CELL_SIZE as f32).floor() as u32;

    // text setup
    let text_alignment = TextAlignment::Left;
    let text_style = TextStyle {
        font_size: 26.,
        ..default()
    };

    info!("{} {}", win.width(), win.height());
    // offset to center the cells inside the window
    let win_x_offset = win.width() % CELL_SIZE as f32;
    let win_y_offset = win.width() % CELL_SIZE as f32;

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        },
        ..default()
    });

    let quad_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let alive_material = materials.add(ColorMaterial::from(Color::Hsla {
        hue: 240.,
        saturation: 0.87,
        lightness: 0.60,
        alpha: 1.,
    }));
    let dead_material = materials.add(ColorMaterial::from(Color::BLACK));

    commands.insert_resource(StateMaterials {
        alive_material: alive_material.clone(),
        dead_material: dead_material.clone(),
    });
    commands.insert_resource(LastUpdate(0.));

    // keeps a 2x2 matrix of all the entities for faster indexing
    let mut entity_map = EntityMap { v: vec![] };

    for i in 0..rows {
        entity_map.v.push(vec![]);
        for j in 0..columns {
            let x_offset = (i * CELL_SIZE) as f32 - (win.width() / 2.0).round()
                + (CELL_SIZE / 2) as f32
                + (win_x_offset / 2.);
            let y_offset = (j * CELL_SIZE) as f32 * -1. + (win.height() / 2.0).round()
                - (CELL_SIZE / 2) as f32
                - (win_y_offset / 2.);

            // let entity_id = commands
            let entity_id = commands
                .spawn(SpriteBundle {
                    texture: asset_server.load("cells/alive_cell.png"),
                    transform: Transform::from_translation(Vec3 {
                        x: x_offset,
                        y: y_offset,
                        z: 1.,
                    }),
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(CELL_SIZE as f32)),
                        color: Color::BLACK,

                        ..Default::default()
                    },
                    ..default()
                })
                .insert(Cell {
                    next_state: None,
                    state: State::DEAD,
                })
                .id();
            let row = entity_map.v.get_mut(i as usize);
            row.unwrap().push(entity_id);
        }
    }

    // Add the entity map to the resources so they can be accessed by the update systems
    commands.insert_resource(entity_map);
    // Add text

    commands
        .spawn(NodeBundle {
            style: Style {
                align_content: AlignContent::FlexStart,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                align_self: AlignSelf::Start,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_section("0", text_style))
                .insert(FPSCounter);
        });
}
