use bevy::{prelude::*, render::extract_resource::ExtractResourcePlugin};

// Inspired by bevy's example
pub struct GameOfLifePlugin;
pub struct GameOfLifeComputePlugin;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin)
    }
}
