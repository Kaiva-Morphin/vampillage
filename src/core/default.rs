pub mod plugin {
    use bevy::{app::Plugin, asset::{AssetMetaCheck, AssetPlugin}, math::vec2, prelude::{default, App, PluginGroup}, render::{texture::ImagePlugin, view::Msaa}, window::{PresentMode, Window, WindowPlugin, WindowTheme}, DefaultPlugins};
    use bevy_easings::EasingsPlugin;
    use bevy_rapier2d::render::RapierDebugRenderPlugin;
    use bevy_rapier2d::prelude::*;

    use crate::core::{camera::plugin::EnhancedCameraPlugin, despawn_lifetime::DespawnLifetimePlugin, functions::TextureAtlasLayoutHandles, post_processing::PostProcessPlugin, ui::UIPlugin};
    pub struct DefaultPlugin;

    impl Plugin for DefaultPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins((
                DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                                present_mode: PresentMode::AutoNoVsync,
                                window_theme: Some(WindowTheme::Dark),
                                canvas: Some("#bevy".to_string()),
                                fit_canvas_to_parent: true,
                                title: "Bloody Night".into(),
                                ..default()
                            }),
                            ..default()
                        }).set(AssetPlugin {
                            // Wasm builds will check for meta files (that don't exist) if this isn't set.
                            // This causes errors and even panics on web build on itch.
                            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                            meta_check: AssetMetaCheck::Never,
                            ..default()
                        }),
                //RapierDebugRenderPlugin::default().disabled(),
                RapierPhysicsPlugin::<NoUserData>::default(),
                //SwitchableRapierDebugPlugin,
                EnhancedCameraPlugin,
                PostProcessPlugin,
                DespawnLifetimePlugin,
                EasingsPlugin,
                UIPlugin
            ),
            );
            app.insert_resource(TextureAtlasLayoutHandles::default());
            app.insert_resource(Msaa::Off);
            app.insert_resource(RapierConfiguration {
                gravity: vec2(0.0, 0.0),
                physics_pipeline_active: true,
                query_pipeline_active: true,
                timestep_mode: TimestepMode::Variable {
                    max_dt: 1.0 / 60.0,
                    time_scale: 1.0,
                    substeps: 1,
                },
                scaled_shape_subdivision: 10,
                force_update_from_transform_changes: false,
            });
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        }
    }
}
/*
app.add_systems(Startup, set_window_icon);

fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>,
) {
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("my_icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();
    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
    */