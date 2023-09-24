use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::render::render_resource::TextureFormat;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use rand::Rng;

////////////////////////////////////////////////////////////////////////
/// COMPONENTS
////////////////////////////////////////////////////////////////////////
#[derive(Component, Debug, Copy, Clone)]
enum State {
    ALIVE,
    DEAD,
}
#[derive(Component)]
struct Board;

////////////////////////////////////////////////////////////////////////
/// RESOURCES
////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Debug, Resource, Copy, Clone)]
pub enum Seed {
    Random,
    Spaceship,
    GosperGliderGun,
    SimkinGliderGun,
}
#[derive(Resource, Debug, Clone, Copy)]
pub struct GameSettings {
    pub cell_size: u8,
    pub time_step_secs: f64,
    pub alive_color: [u8; 4],
    pub dead_color: [u8; 4],
    pub seed: Seed,
}
#[derive(Resource, Debug)]
pub struct Brush {
    pub size: u8,
}
#[derive(Resource)]
struct BoardHandle(Handle<Image>);
#[derive(Resource, Debug)]
struct BoardSize {
    rows: u32,
    columns: u32,
}
#[derive(Resource, Debug, Clone)]
struct LastUpdate(f64);

impl Default for Seed {
    fn default() -> Self {
        Seed::Random
    }
}
impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            cell_size: 3,
            time_step_secs: 0.05,
            alive_color: [64, 64, 243, 255],
            dead_color: [0, 0, 0, 255],
            seed: Seed::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////
/// MAIN
////////////////////////////////////////////////////////////////////////

pub fn init() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::default(),
                        present_mode: PresentMode::AutoNoVsync,

                        canvas: Some("#my-canvas".to_string()),
                        fit_canvas_to_parent: true,
                        title: String::from("Conway's game of life"),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            crate::ui::GameOfLifeUI::default(),
        ))
        .insert_resource(GameSettings::default())
        .add_systems(Startup, setup)
        .add_systems(PostStartup, seed) // commands need to be flushed
        .add_systems(
            Update,
            (
                (process_cells).run_if(should_next_tick),
                (handle_events).after(process_cells),
            ),
        )
        .run();
}

////////////////////////////////////////////////////////////////////////
/// RUN CONDITIONS
////////////////////////////////////////////////////////////////////////

// Decides if the `evolution` systems run
fn should_next_tick(
    settings: Res<GameSettings>,
    mut previous_tick: Local<f64>,
    time: Res<Time>,
) -> bool {
    let time_step = settings.time_step_secs;
    if time.elapsed_seconds_f64() - (*previous_tick) >= time_step {
        *previous_tick = time.elapsed_seconds_f64();
        true
    } else {
        false
    }
}
////////////////////////////////////////////////////////////////////////
/// SYSTEMS
///////////////////////////////////////////////////////////////////////

// Creates the entities and resources
fn setup(
    mut commands: Commands,
    q_win: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
    settings: Res<GameSettings>,
) {
    let win = q_win.single();
    let rows = (win.width() / settings.cell_size as f32).floor() as u32;
    let columns = (win.height() / settings.cell_size as f32).floor() as u32;
    let board = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: rows,
            height: columns,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &(settings.dead_color.clone()),
        TextureFormat::Rgba8Unorm,
    );
    // text setup
    let image = images.add(board);

    // Initialize resources
    commands.insert_resource(BoardHandle(image.clone()));
    commands.insert_resource(BoardSize { rows, columns });
    commands.insert_resource(Brush { size: 1 });
    commands.insert_resource(LastUpdate(0.));

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        },
        ..default()
    });

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(win.width() as f32, win.height() as f32)),

                ..default()
            },
            texture: image.clone(),
            ..default()
        })
        .insert(Board);
}

// Updates the next_state of the cells and after all the cells have been updated, state=next_state
fn process_cells(
    mut images: ResMut<Assets<Image>>,
    board_handle: Res<BoardHandle>,
    board_size: Res<BoardSize>,
    mut next_state: Local<Vec<u8>>,
    settings: Res<GameSettings>,
) {
    let h = &board_handle.0;

    if let Some(board) = images.get_mut(h) {
        if next_state.len() != board.data.len() {
            // Initialize the buffer containing the next state
            *next_state = board.data.clone();
        }
        for i in 0..(board.data.len() / 4) {
            // component
            let c = i * 4;

            let y = (i as f32 / board_size.rows as f32).floor() as i32;
            let x = i as i32 - y * board_size.rows as i32;

            let new_cell_state = cell_state(&board, x, y, &settings);
            match new_cell_state {
                State::ALIVE => {
                    next_state[c + 0] = settings.alive_color[0];
                    next_state[c + 1] = settings.alive_color[1];
                    next_state[c + 2] = settings.alive_color[2];
                    next_state[c + 3] = settings.alive_color[3];
                }
                State::DEAD => {
                    next_state[c + 0] = settings.dead_color[0];
                    next_state[c + 1] = settings.dead_color[1];
                    next_state[c + 2] = settings.dead_color[2];
                    next_state[c + 3] = settings.dead_color[3];
                }
            }
        }
        board.data = next_state.clone();
    };
}

fn handle_events(
    mut resize_events: EventReader<WindowResized>,
    mut board_sprite: Query<&mut Sprite, With<Board>>,
    q_win: Query<&Window, With<PrimaryWindow>>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    board_size: Res<BoardSize>,
    mut images: ResMut<Assets<Image>>,
    mut brush: ResMut<Brush>,
    board_handle: Res<BoardHandle>,
    mut exit: EventWriter<bevy::app::AppExit>,
    settings: Res<GameSettings>,
) {
    // Resize the board sprite if the window's size has changed
    for resize in resize_events.iter() {
        let mut board = board_sprite.single_mut();
        board.custom_size = Some(Vec2::new(resize.width, resize.height));
    }
    // J: makes the brush size smaller
    if keys.pressed(KeyCode::J) {
        if brush.size > 1 {
            brush.size -= 1;
        }
    }
    // J: makes the brush size smaller
    if keys.pressed(KeyCode::K) {
        if brush.size < u8::MAX {
            brush.size += 1;
        }
    }
    // Exit the app if we press Esc
    if keys.pressed(KeyCode::Escape) {
        exit.send(bevy::app::AppExit);
    }

    // We'll add a living cell on the point where mouse was pressed
    if buttons.pressed(MouseButton::Left) {
        let win = q_win.single();
        if let Some(position) = win.cursor_position() {
            // X in the texture buffer
            let posx = (position.x / win.width() * board_size.rows as f32).round() as i32;
            let posy = (position.y / win.height() * board_size.columns as f32).round() as i32;

            let h = &board_handle.0;

            if let Some(board) = images.get_mut(h) {
                // We iterate through the square of the brush, we check if the pixel we picked is within the range of the circle around our cursor
                for bx in -(brush.size as i32)..=brush.size as i32 {
                    for by in -(brush.size as i32)..=brush.size as i32 {
                        let x = posx + bx;
                        let y = posy + by;

                        let r = (((x - posx).pow(2) + (y - posy).pow(2)) as f32).sqrt();
                        if r <= brush.size as f32 {
                            // this probably can be done in a more rusty way instead of just raw pointers but the borrow checker wont let you do [&mut u8; 4] obv ~
                            if let Some(pixel) = board.get_pixel_mut(x, y) {
                                unsafe {
                                    *pixel[0] = settings.alive_color[0];
                                    *pixel[1] = settings.alive_color[1];
                                    *pixel[2] = settings.alive_color[2];
                                    *pixel[3] = settings.alive_color[3];
                                }
                            };
                        }
                    }
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////
/// UTILS
////////////////////////////////////////////////////////////////////////
// Looks at a cell at a pixel in the image and determines if it's alive
fn cell_state(image: &Image, x: i32, y: i32, settings: &GameSettings) -> State {
    // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules

    let mut neighbours_alive = 0;

    // neighbours x
    for nx in -1..=1 {
        // neighbors y
        for ny in -1..=1 {
            // if its the center one (the cell we're determining)
            if nx == 0 && ny == 0 {
                continue;
            }

            let n = image.get_pixel(x + nx, y + ny);
            if let Some(n_cell) = n {
                match State::cell_state(&n_cell, settings) {
                    State::ALIVE => neighbours_alive += 1,
                    State::DEAD => {}
                }
            }
        }
    }

    let cell_state = State::cell_state(&image.get_pixel(x, y).unwrap(), settings);

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

// Seeds the state of the board (for now just a simple 50%)
fn seed(
    mut images: ResMut<Assets<Image>>,
    board_handle: Res<BoardHandle>,
    settings: Res<GameSettings>,
) {
    let h = &board_handle.0;

    let mut rng = rand::thread_rng();

    if let Some(board) = images.get_mut(h) {
        for i in 0..(board.data.len() / 4) {
            let rand: f32 = rng.gen();
            if rand >= 0.5 {
                board.data[i * 4 + 0] = settings.alive_color[0];
                board.data[i * 4 + 1] = settings.alive_color[1];
                board.data[i * 4 + 2] = settings.alive_color[2];
                board.data[i * 4 + 3] = settings.alive_color[3];
            }
        }
    }
}

trait Pixel {
    fn get_pixel(&self, x: i32, y: i32) -> Option<[&u8; 4]>;
    fn get_pixel_mut(&mut self, x: i32, y: i32) -> Option<[*mut u8; 4]>;
}
impl Pixel for Image {
    fn get_pixel(&self, x: i32, y: i32) -> Option<[&u8; 4]> {
        let size = self.size();

        if x >= size.x as i32 || x < 0 as i32 || y >= size.y as i32 || y < 0 {
            return None;
        }

        let pos = (y * size.x as i32 + x) * 4;

        let r = &self.data[(pos + 0) as usize];
        let g = &self.data[(pos + 1) as usize];
        let b = &self.data[(pos + 2) as usize];
        let a = &self.data[(pos + 3) as usize];

        return Some([r, g, b, a]);
    }
    fn get_pixel_mut(&mut self, x: i32, y: i32) -> Option<[*mut u8; 4]> {
        let size = self.size();

        if x >= size.x as i32 || x < 0 as i32 || y >= size.y as i32 || y < 0 {
            return None;
        }

        let pos = (y * size.x as i32 + x) * 4;

        let r = unsafe { self.data.as_mut_ptr().add(pos as usize + 0) };
        let g = unsafe { self.data.as_mut_ptr().add(pos as usize + 1) };
        let b = unsafe { self.data.as_mut_ptr().add(pos as usize + 2) };
        let a = unsafe { self.data.as_mut_ptr().add(pos as usize + 3) };

        return Some([r, g, b, a]);
    }
}
impl State {
    fn cell_state(data: &[&u8; 4], settings: &GameSettings) -> State {
        // cells are red
        if *data[0] == settings.alive_color[0]
            && *data[1] == settings.alive_color[1]
            && *data[2] == settings.alive_color[2]
            && *data[3] == settings.alive_color[3]
        {
            State::ALIVE
        } else {
            State::DEAD
        }
    }
}
