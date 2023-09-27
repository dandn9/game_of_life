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

use crate::ui::UIEvent;

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
    pub time_step_secs: f32,
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

impl Default for Seed {
    fn default() -> Self {
        Seed::Random
    }
}
impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            cell_size: 3,
            time_step_secs: 0.03,
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
        .add_systems(Update, process_cells)
        .add_systems(Last, (handle_ui_events, handle_events))
        .run();
}

////////////////////////////////////////////////////////////////////////
/// RUN CONDITIONS
////////////////////////////////////////////////////////////////////////

// Decides if the `evolution` systems run
// fn should_next_tick(
//     settings: Res<GameSettings>,
//     mut previous_tick: Local<f64>,
//     time: Res<Time>,
// ) -> bool {
//     let time_step = settings.time_step_secs;
//     if time.elapsed_seconds_f64() - (*previous_tick) >= time_step {
//         *previous_tick = time.elapsed_seconds_f64();
//         true
//     } else {
//         false
//     }
// }
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
    let board_settings = create_board(&settings, &win);
    let mut board = board_settings.0;
    let rows = board_settings.1;
    let columns = board_settings.2;

    // Seed the board
    seed(&mut board, &settings);
    // text setup
    let image = images.add(board);

    // Initialize resources
    commands.insert_resource(BoardHandle(image.clone()));
    commands.insert_resource(BoardSize { rows, columns });
    commands.insert_resource(Brush { size: 1 });

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
fn create_board(settings: &GameSettings, win: &Window) -> (Image, u32, u32) {
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
    (board, rows, columns)
}

// Updates the next_state of the cells and after all the cells have been updated, state=next_state
fn process_cells(
    mut images: ResMut<Assets<Image>>,
    board_handle: Res<BoardHandle>,
    board_size: Res<BoardSize>,
    mut next_state: Local<Vec<u8>>,
    settings: Res<GameSettings>,
    mut previous_tick: Local<f64>,
    time: Res<Time>,
) {
    // Check in the system since run conditions mess up with the scheduling
    let time_step = settings.time_step_secs;
    if time.elapsed_seconds_f64() - (*previous_tick) <= time_step as f64 {
        return ();
    }
    *previous_tick = time.elapsed_seconds_f64();
    let h = &board_handle.0;

    let board = images.get_mut(h).unwrap();
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
}

// // Events triggered by the ui
fn handle_ui_events(
    mut ui_events: EventReader<UIEvent>,
    mut images: ResMut<Assets<Image>>,
    mut board_handle: ResMut<BoardHandle>,
    mut settings: ResMut<GameSettings>,

    mut texture: Query<&mut Handle<Image>, With<Board>>, // The handle to the board's texture
    mut commands: Commands,
    q_win: Query<&Window>,
    mut board_size: ResMut<BoardSize>,
) {
    for ev in ui_events.iter() {
        match *ev {
            UIEvent::ChangeColor(alive_color, dead_color) => {
                info!("CHANGE COLOR {:?} {:?}", alive_color, dead_color);
                let h = &board_handle.0;
                let board = images.get_mut(h).unwrap();
                // let mut data = board.data;
                for i in 0..(board.data.len() / 4) {
                    let c = i * 4;
                    let state = State::cell_state(
                        &[
                            &board.data[c + 0],
                            &board.data[c + 1],
                            &board.data[c + 2],
                            &board.data[c + 3],
                        ],
                        &settings,
                    );
                    match state {
                        State::ALIVE => {
                            board.data[c + 0] = alive_color[0];
                            board.data[c + 1] = alive_color[1];
                            board.data[c + 2] = alive_color[2];
                            board.data[c + 3] = alive_color[3];
                        }
                        State::DEAD => {
                            board.data[c + 0] = dead_color[0];
                            board.data[c + 1] = dead_color[1];
                            board.data[c + 2] = dead_color[2];
                            board.data[c + 3] = dead_color[3];
                        }
                    }
                }
                settings.alive_color = alive_color;
                settings.dead_color = dead_color;
            }
            UIEvent::ChangeSeed(seed_value) => {
                let h = &board_handle.0;
                let board = images.get_mut(h).unwrap();
                settings.seed = seed_value;
                reset_board(board, &settings);
                seed(board, &settings);
            }
            UIEvent::ChangeTimestep(time_step) => {
                settings.time_step_secs = time_step;
            }
            UIEvent::ChangeCellSize(cell_size) => {
                images.remove(&board_handle.0);
                settings.cell_size = cell_size;
                let window = q_win.single();
                let mut new_board = create_board(&settings, &window);
                seed(&mut new_board.0, &settings);
                let image_handle = images.add(new_board.0);
                let mut texture = texture.single_mut();

                *texture = image_handle.clone();
                *board_handle = BoardHandle(image_handle.clone());
                *board_size = BoardSize {
                    rows: new_board.1,
                    columns: new_board.2,
                };
            }

            _ => {}
        }
    }
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
    mut eguic: bevy_egui::EguiContexts,
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
        let eguictx = eguic.ctx_mut();
        // Skip the event if mouse is over UI element
        if eguictx.is_pointer_over_area() {
            return ();
        }

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

// Puts the whole board on dead
fn reset_board(board: &mut Image, settings: &GameSettings) {
    for i in 0..board.data.len() / 4 {
        let c = i * 4;
        board.data[c + 0] = settings.dead_color[0];
        board.data[c + 1] = settings.dead_color[1];
        board.data[c + 2] = settings.dead_color[2];
        board.data[c + 3] = settings.dead_color[3];
    }
}
// Seeds the state of the board (for now just a simple 50%)
fn seed(board: &mut Image, settings: &GameSettings) {
    // match settings.seed

    let gun_to_buff_values = |settings: &GameSettings, gun: &Vec<Vec<i32>>| {
        let mut res: Vec<Vec<u8>> = vec![];
        for i in 0..gun.len() {
            let row = &gun[i];
            let mut r = vec![];
            for j in 0..row.len() {
                if row[j] == 1 {
                    r.push(settings.alive_color[0]);
                    r.push(settings.alive_color[1]);
                    r.push(settings.alive_color[2]);
                    r.push(settings.alive_color[3]);
                } else {
                    r.push(settings.dead_color[0]);
                    r.push(settings.dead_color[1]);
                    r.push(settings.dead_color[2]);
                    r.push(settings.dead_color[3]);
                }
            }
            res.push(r);
        }
        return res;
    };
    match settings.seed {
        Seed::Random => {
            let mut rng = rand::thread_rng();
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
        // TODO: make this better XD
        Seed::GosperGliderGun => {
            // https://upload.wikimedia.org/wikipedia/commons/thumb/e/e0/Game_of_life_glider_gun.svg/500px-Game_of_life_glider_gun.svg.png
            #[rustfmt::skip]
            let glider_gun = vec![
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1],
                vec![0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1],
                vec![1,1,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![1,1,0,0,0,0,0,0,0,0,1,0,0,0,1,0,1,1,0,0,0,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            ];

            let board_size = board.size();

            let values = gun_to_buff_values(settings, &glider_gun);

            #[rustfmt::skip]
            let offset_y = if glider_gun.len() as f32 % 2. != 0. { 1 } else { 0 };
            #[rustfmt::skip]
            let offset_x = if glider_gun[0].len() as f32 % 2. != 0. { 1 } else { 0 };

            let center = board_size / 2.;
            let x_start = center.x as usize - glider_gun[0].len() / 2;
            let x_end = (center.x as usize + glider_gun[0].len() / 2) + offset_x;
            let y_start = center.y as usize - glider_gun.len() / 2;
            let y_end = (center.y as usize + glider_gun.len() / 2) + offset_y;

            let mut yy = 0;
            let mut xx = 0;
            for i in y_start..y_end {
                for j in x_start..x_end {
                    let offset = (i * board_size.x as usize * 4) + j * 4;
                    board.data[offset + 0] = values[yy][xx + 0];
                    board.data[offset + 1] = values[yy][xx + 1];
                    board.data[offset + 2] = values[yy][xx + 2];
                    board.data[offset + 3] = values[yy][xx + 3];
                    xx += 4;
                }
                xx = 0;
                yy += 1;
            }
        }
        Seed::SimkinGliderGun => {
            #[rustfmt::skip]
            let glider_gun = vec![
                vec![1,1,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![1,1,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,1,1,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,1,1],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,0,0,1,0,0,0,1,1],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,0,0,0,0],
                vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,1,0,0,0,0,0],
            ];

            let board_size = board.size();

            let values = gun_to_buff_values(settings, &glider_gun);

            #[rustfmt::skip]
            let offset_y = if glider_gun.len() as f32 % 2. != 0. { 1 } else { 0 };
            #[rustfmt::skip]
            let offset_x = if glider_gun[0].len() as f32 % 2. != 0. { 1 } else { 0 };

            let center = board_size / 2.;
            let x_start = center.x as usize - glider_gun[0].len() / 2;
            let x_end = (center.x as usize + glider_gun[0].len() / 2) + offset_x;
            let y_start = center.y as usize - glider_gun.len() / 2;
            let y_end = (center.y as usize + glider_gun.len() / 2) + offset_y;

            let mut yy = 0;
            let mut xx = 0;
            for i in y_start..y_end {
                for j in x_start..x_end {
                    let offset = (i * board_size.x as usize * 4) + j * 4;
                    board.data[offset + 0] = values[yy][xx + 0];
                    board.data[offset + 1] = values[yy][xx + 1];
                    board.data[offset + 2] = values[yy][xx + 2];
                    board.data[offset + 3] = values[yy][xx + 3];
                    xx += 4;
                }
                xx = 0;
                yy += 1;
            }
        }

        _ => {}
    }
}
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
