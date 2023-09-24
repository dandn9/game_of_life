use bevy::{
    a11y::accesskit::TextSelection,
    prelude::*,
    render::extract_resource::{ExtractResource, ExtractResourcePlugin},
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self, Align2, ComboBox, Pos2, WidgetText},
    EguiContexts, EguiPlugin,
};

use crate::game_of_life::GameSettings;

/**
 *  This plugin is responsible for the UI of the game
 */

#[derive(Resource)]
pub struct GameOfLifeUI {
    show: bool,
}
impl Default for GameOfLifeUI {
    fn default() -> Self {
        GameOfLifeUI { show: true }
    }
}

#[derive(Component)]
struct FPSCounter;

#[derive(Component)]
struct UI;

impl Plugin for GameOfLifeUI {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameOfLifeUI::default())
            .add_plugins(bevy_egui::EguiPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    egui_init,
                    handle_events,
                    (update_fps_counter).run_if(should_update_counter(0.1)),
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

fn egui_init(
    mut eguic: EguiContexts,
    q_win: Query<&Window, With<PrimaryWindow>>,
    ui_state: Res<GameOfLifeUI>,
    mut settings: ResMut<GameSettings>,
) {
    if ui_state.show {
        egui::Window::new("id")
            .auto_sized()
            .anchor(Align2::RIGHT_TOP, egui::vec2(0., 0.))
            .movable(false)
            .show(eguic.ctx_mut(), |ui| {
                // let mut selected = Seed::Default;

                // let a = ui.add(egui::ComboBox::from_label("SelectOne!").show_ui(ui, |ui| {
                //     ui.selectable_value(&mut selected, Enum::First, "First");
                //     ui.selectable_value(&mut selected, Enum::Second, "Second");
                //     ui.selectable_value(&mut selected, Enum::Third, "Third");
                // }));

                // ui.add(ComboBox::new(23, "xd"));

                let cell_size = &mut settings.cell_size;
                ui.add(
                    egui::Slider::new(cell_size, 1..=30)
                        .step_by(1.0)
                        .text("Cell Size"),
                );

                let mut selected = settings.seed.clone();
                let mut egui_color: [f32; 4] = [
                    settings.alive_color[0] as f32 / 255.,
                    settings.alive_color[1] as f32 / 255.,
                    settings.alive_color[2] as f32 / 255.,
                    settings.alive_color[3] as f32 / 255.,
                ];
                ui.color_edit_button_rgba_unmultiplied(&mut egui_color);
                // Select with all the possible seeds
                egui::ComboBox::from_label("Seed")
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut selected,
                            crate::game_of_life::Seed::Random,
                            "Random",
                        );
                        ui.selectable_value(
                            &mut selected,
                            crate::game_of_life::Seed::Spaceship,
                            "Spaceship",
                        );
                        ui.selectable_value(
                            &mut selected,
                            crate::game_of_life::Seed::GosperGliderGun,
                            "Gosper Glider Gun",
                        );
                        ui.selectable_value(
                            &mut selected,
                            crate::game_of_life::Seed::SimkinGliderGun,
                            "Simking Glider Gun",
                        );
                    });

                settings.seed = selected;
                settings.alive_color = [
                    (egui_color[0] * 255.).round() as u8,
                    (egui_color[1] * 255.).round() as u8,
                    (egui_color[2] * 255.).round() as u8,
                    (egui_color[3] * 255.).round() as u8,
                ];

                // info!("SELECTED {:?}", selected);
                ui.label("YO");
            });
    }
    // let a = .show(eguic.ctx_mut(), |ui| {
    //     ui.heading("My egui Application");
    //     ui.horizontal(|ui| {
    //         let name_label = ui.label("Your name: ");
    //         ui.text_edit_singleline(&mut String::from("hi"))
    //             .labelled_by(name_label.id);
    //     });
    //     ui.add(egui::Slider::new(&mut 0, 0..=120).text("age"));
    //     if ui.button("Click each year").clicked() {
    //         // self.age += 1;
    //     }
    //     ui.label(format!("Hello '{}', age {}", "xd", 5));
    // });
}
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
    settings: Res<GameSettings>,
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
                brush.size as f32 * settings.cell_size as f32,
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
