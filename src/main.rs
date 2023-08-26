use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, seed) // commands need to be flushed
        .add_systems(Update, (process_cells, update_colors.after(process_cells)))
        .run();
}

#[derive(Component, Debug, Copy, Clone)]
enum State {
    ALIVE,
    DEAD,
}
#[derive(Component, Debug)]
struct Cell {
    state: State,
    next_state: Option<State>,
    position: Vec2,
}
#[derive(Resource, Debug)]
struct EntityMap {
    v: Vec<Vec<Entity>>,
}

#[derive(Resource)]
struct StateMaterials {
    alive_material: Handle<ColorMaterial>,
    dead_material: Handle<ColorMaterial>,
}

#[derive(Resource)]
struct LastUpdate(f64);

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

const TIME_STEP_SECS: f64 = 0.5;
fn process_cells(
    time: Res<Time>,
    mut cells: Query<&mut Cell>,
    entities_map: ResMut<EntityMap>,
    mut last_update: ResMut<LastUpdate>,
) {
    if time.elapsed_seconds_f64() - last_update.0 >= TIME_STEP_SECS {
        last_update.0 = time.elapsed_seconds_f64();
    } else {
        return;
    }

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

fn seed(mut query: Query<&mut Cell>) {
    for mut cell in query.iter_mut() {
        println!(
            "X POSITION {} Y POSITION {}",
            cell.position.x, cell.position.y
        );
        if cell.position.x > 0.
            && cell.position.x < 20.
            && cell.position.y > 10.
            && cell.position.y < 20.
        {
            cell.state = State::ALIVE;
        }
    }
}

fn update_colors(
    mut commands: Commands,
    mut query: Query<(Entity, &Cell, &mut Handle<ColorMaterial>)>,
    state_materials: Res<StateMaterials>,
) {
    for (e, cell, mut material) in query.iter_mut() {
        match cell.state {
            State::ALIVE => {
                *material = state_materials.alive_material.clone();
            }
            State::DEAD => {
                *material = state_materials.dead_material.clone();
            }
        }
    }
}

const CELL_SIZE: u32 = 20;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    win_q: Query<&Window>,
) {
    let win = win_q.single();
    let rows = (win.width() / CELL_SIZE as f32).floor() as u32;
    let columns = (win.height() / CELL_SIZE as f32).floor() as u32;

    // offset to center the cells inside the window
    let win_x_offset = win.width() % CELL_SIZE as f32;
    let win_y_offset = win.width() % CELL_SIZE as f32;

    commands.spawn(Camera2dBundle::default());

    let quad_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    let alive_material = materials.add(ColorMaterial::from(Color::RED));
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

            let entity_id = commands
                .spawn(MaterialMesh2dBundle {
                    mesh: quad_mesh.clone().into(),
                    transform: Transform {
                        translation: Vec3::new(x_offset, y_offset, 1.0),
                        scale: Vec3::splat((CELL_SIZE - 2) as f32),
                        ..Transform::default()
                    },
                    material: dead_material.clone(),
                    ..default()
                })
                .insert(Cell {
                    next_state: None,
                    state: State::DEAD,
                    position: Vec2 {
                        x: i as f32,
                        y: j as f32,
                    },
                })
                .id();
            let row = entity_map.v.get_mut(i as usize);
            row.unwrap().push(entity_id);
        }
    }

    // Add the entity map to the resources so they can be accessed by the update systems
    commands.insert_resource(entity_map);
}
