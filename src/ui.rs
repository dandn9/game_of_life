use bevy::{
    a11y::accesskit::TextSelection, prelude::*, render::extract_resource::ExtractResourcePlugin,
    sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};

/**
 *  This plugin is responsible for the UI of the game
 */

#[derive(Default, Resource)]
pub struct GameOfLifeUI {
    show: bool,
}

#[derive(Component)]
struct FPSCounter;

#[derive(Component)]
struct UI;

impl Plugin for GameOfLifeUI {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameOfLifeUI::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    handle_events,
                    (update_fps_counter).run_if(should_update_counter(1.)),
                    (update_ui_visibility),
                ),
            );
        // Update the fps counter every 1 second
    }
}

fn setup(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 26.,
        ..default()
    };
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
        .insert(UI)
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_section("0", text_style))
                .insert(FPSCounter);
        });
}

////////////////////////////////////////////////////////////////////////
/// SYSTEMS
////////////////////////////////////////////////////////////////////////

fn handle_events(keys: Res<Input<KeyCode>>, mut ui_state: ResMut<GameOfLifeUI>) {
    // Toggle the ui if U is pressed
    if keys.just_pressed(KeyCode::U) {
        ui_state.show = !ui_state.show
    }
}

fn update_fps_counter(time: Res<Time>, mut counter: Query<&mut Text, With<FPSCounter>>) {
    let delta_time = time.delta_seconds_f64();
    let fps = (1. / delta_time) as i32;

    let mut text = counter.single_mut();
    if let Some(section) = text.sections.first_mut() {
        section.value = fps.to_string();
    }
}

fn update_ui_visibility(
    ui_state: Res<GameOfLifeUI>,
    mut ui_elems: Query<&mut Visibility, With<UI>>,
    mut gizmos: Gizmos,
    q_win: Query<&Window, With<PrimaryWindow>>,
    brush: Res<crate::game_of_life::Brush>,
) {
    for mut ui_elem in ui_elems.iter_mut() {
        if ui_state.show {
            *ui_elem = Visibility::Visible;
        } else {
            *ui_elem = Visibility::Hidden
        }
    }

    // Draw a gizmo on top of the cursor displaying the brush size
    if ui_state.show {
        let win = q_win.single();
        if let Some(cursor) = win.cursor_position() {
            let w = win.width();
            let h = win.height();
            let circle_pos = (cursor - Vec2::new(w / 2., h / 2.)) * Vec2::new(1., -1.);
            gizmos.circle_2d(
                circle_pos,
                brush.size as f32 * crate::game_of_life::CELL_SIZE as f32,
                Color::WHITE,
            );
        }
    }
}

////////////////////////////////////////////////////////////////////////
/// RUN CONDITIONS
////////////////////////////////////////////////////////////////////////

fn should_update_visibility(ui: Res<GameOfLifeUI>) -> bool {
    if ui.is_changed() {
        true
    } else {
        false
    }
}
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
