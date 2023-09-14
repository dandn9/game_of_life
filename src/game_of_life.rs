use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::render_resource::TextureFormat;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use rand::Rng;

const CELL_SIZE: u32 = 5;
const TIME_STEP_SECS: f64 = 0.5;

const ALIVE_COLOR: [u8; 4] = [255, 0, 0, 255];
const DEAD_COLOR: [u8; 4] = [10, 10, 10, 255];

////////////////////////////////////////////////////////////////////////
/// COMPONENTS
///////////////////////////////////////////////////////////////////////

#[derive(Component, Debug, Copy, Clone)]
enum State {
    ALIVE,
    DEAD,
}

impl State {
    fn get_alive_color() -> [u8; 4] {
        [255, 0, 0, 255]
    }
    fn cell_state(data: &[&u8; 4]) -> State {
        // cells are red
        if *data[0] == ALIVE_COLOR[0]
            && *data[1] == ALIVE_COLOR[1]
            && *data[2] == ALIVE_COLOR[2]
            && *data[3] == ALIVE_COLOR[3]
        {
            State::ALIVE
        } else {
            State::DEAD
        }
    }
}

#[derive(Component, Debug)]
struct Cell {
    state: State,
}

#[derive(Component)]
struct FPSCounter;

#[derive(Resource, Debug, Clone)]
struct LastUpdate(f64);

////////////////////////////////////////////////////////////////////////

pub fn main() {
    // console_log::init_with_level(Level::DEBUG);

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::default(),
                    present_mode: PresentMode::AutoNoVsync,

                    canvas: Some("#my-canvas".to_string()),
                    fit_canvas_to_parent: true,

                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, seed) // commands need to be flushed
        .add_systems(
            Update,
            (
                (process_cells).run_if(should_next_tick(TIME_STEP_SECS)),
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

trait Pixel {
    fn get_pixel(&self, x: i32, y: i32) -> Option<[&u8; 4]>;
}
impl Pixel for Image {
    fn get_pixel(&self, x: i32, y: i32) -> Option<[&u8; 4]> {
        let size = self.size();

        if x > size.x as i32 || x < 0 as i32 || y > size.y as i32 || y < 0 {
            return None;
        }
        let r = &self.data[(y * size.x as i32 + x + 0) as usize];
        let g = &self.data[(y * size.x as i32 + x + 1) as usize];
        let b = &self.data[(y * size.x as i32 + x + 2) as usize];
        let a = &self.data[(y * size.x as i32 + x + 3) as usize];

        return Some([r, g, b, a]);
    }
}

// Looks at a cell at a pixel in the image and determines if it's alive
fn cell_state(image: &Image, x: i32, y: i32) -> State {
    // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules

    let mut neighbours_alive = 0;

    // neighbours x
    for n_x in -1..=1 {
        // neighbors y
        for n_y in -1..=1 {
            // if its the center one (the cell we're determining)
            if n_x == 0 && n_y == 0 {
                continue;
            }

            let n = image.get_pixel(x + n_x, y + n_y);
            if let Some(n_cell) = n {
                match State::cell_state(&n_cell) {
                    State::ALIVE => neighbours_alive += 1,
                    State::DEAD => {}
                }
            }
        }
    }

    let cell_state = State::cell_state(&image.get_pixel(x, y).unwrap());

    match cell_state {
        State::ALIVE => {
            if neighbours_alive < 2 {
                return State::DEAD;
            }
            if neighbours_alive == 2 || neighbours_alive == 3 {
                return State::ALIVE;
            } else {
                return State::DEAD;
            };
        }
        State::DEAD => {
            if neighbours_alive == 3 {
                return State::ALIVE;
            } else {
                return State::DEAD;
            }
        }
    }
}

// Updates the next_state of the cells and after all the cells have been updated, state=next_state
fn process_cells(
    mut images: ResMut<Assets<Image>>,
    board_handle: Res<BoardHandle>,
    board_size: Res<BoardSize>,
) {
    let h = &board_handle.0;

    if let Some(board) = images.get_mut(h) {
        let mut new_state = board.data.clone();

        for i in 0..(board.data.len() / 4) {
            // component
            let c = i * 4;

            let y = (i as f32 / board_size.rows as f32).floor() as i32;
            let x = i as i32 - y * board_size.rows as i32;

            let new_cell_state = cell_state(&board, x, y);
            match new_cell_state {
                State::ALIVE => {
                    new_state[c + 0] = ALIVE_COLOR[0];
                    new_state[c + 1] = ALIVE_COLOR[1];
                    new_state[c + 2] = ALIVE_COLOR[2];
                    new_state[c + 3] = ALIVE_COLOR[3];
                }
                State::DEAD => {
                    new_state[c + 0] = DEAD_COLOR[0];
                    new_state[c + 1] = DEAD_COLOR[1];
                    new_state[c + 2] = DEAD_COLOR[2];
                    new_state[c + 3] = DEAD_COLOR[3];
                }
            }
        }
        board.data = new_state;
    };
}

// Seeds the state of the board (for now just a simple 50%)
fn seed(mut images: ResMut<Assets<Image>>, board_handle: Res<BoardHandle>) {
    let h = &board_handle.0;

    let mut rng = rand::thread_rng();

    if let Some(board) = images.get_mut(h) {
        for i in 0..(board.data.len() / 4) {
            let rand: f32 = rng.gen();
            if rand >= 0.1 {
                board.data[i * 4 + 0] = ALIVE_COLOR[0];
                board.data[i * 4 + 1] = ALIVE_COLOR[1];
                board.data[i * 4 + 2] = ALIVE_COLOR[2];
                board.data[i * 4 + 3] = ALIVE_COLOR[3];
            }
        }
    }
}

#[derive(Resource)]
struct BoardHandle(Handle<Image>);
#[derive(Resource)]
struct BoardSize {
    rows: u32,
    columns: u32,
}
// Creates the entities and resources
fn setup(mut commands: Commands, win_q: Query<&Window>, mut images: ResMut<Assets<Image>>) {
    let win = win_q.single();
    let rows = (win.width() / CELL_SIZE as f32).floor() as u32;
    let columns = (win.height() / CELL_SIZE as f32).floor() as u32;

    let board = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: rows,
            height: columns,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[50, 50, 50, 255],
        TextureFormat::Rgba8Unorm,
    );
    // text setup
    let text_style = TextStyle {
        font_size: 26.,
        ..default()
    };
    let image = images.add(board);
    commands.insert_resource(BoardHandle(image.clone()));
    commands.insert_resource(BoardSize { rows, columns });

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        },
        ..default()
    });

    commands.insert_resource(LastUpdate(0.));
    // keeps a 2x2 matrix of all the entities for faster indexing

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            // custom_size: Some(Vec2::new(win.width() as f32, win.height() as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });

    // UI - Add Text for fps counter
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
