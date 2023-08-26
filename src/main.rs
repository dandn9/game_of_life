use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
};

fn main() {
    App::new()

        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(PostStartup, seed) // commands need to be flushed
        .add_systems(Update, update_colors)
        .run();
}

#[derive(Component)]
enum State {
    ALIVE,
    DEAD,
}
#[derive(Component)]
struct Cell {
    state: State,
    next_state: Option<State>,
    position: Vec2,
}

#[derive(Resource)]
struct StateMaterials {
    alive_material: Handle<ColorMaterial>,
    dead_material: Handle<ColorMaterial>,
}

fn seed(mut query: Query<&mut Cell>) {
    for mut cell in query.iter_mut() {
        println!("X POSITION {} Y POSITION {}", cell.position.x, cell.position.y);
        if cell.position.x > 10.
            && cell.position.x < 20.
            && cell.position.y > 10.
            && cell.position.y < 20.
        {
            cell.state = State::ALIVE;
        }
    }
}

fn update_colors(mut commands: Commands, mut query: Query<(Entity, &Cell, &mut Handle<ColorMaterial>)>, state_materials: Res<StateMaterials>) {
    for (e, cell, mut material) in query.iter_mut() {
        match cell.state {
            State::ALIVE => {
                *material = state_materials.alive_material.clone();
            },
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
        dead_material: dead_material.clone()
    });

    for i in 0..rows {
        for j in 0..columns {
            let x_offset = (i * CELL_SIZE) as f32 - (win.width() / 2.0).round()
                + (CELL_SIZE / 2) as f32
                + (win_x_offset / 2.);
            let y_offset = (j * CELL_SIZE) as f32 * -1. + (win.height() / 2.0).round()
                - (CELL_SIZE / 2) as f32
                - (win_y_offset / 2.);

            // .spawn(MaterialMesh2dBundle {
            //     mesh: quad_mesh.clone().into(),
            //     transform: Transform {
            //         translation: Vec3::new(x_offset, y_offset, 1.0),
            //         scale: Vec3::splat((CELL_SIZE - 2) as f32),
            //         ..Transform::default()
            //     },
            //     material: material.clone(),
            //     ..default()
            // })
            commands
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
                });
        }
    }
}

