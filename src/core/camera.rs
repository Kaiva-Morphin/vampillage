pub mod plugin{

use std::{cell::RefCell, default, sync::Mutex};

use bevy::{core_pipeline::{bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings}, motion_blur::{MotionBlur, MotionBlurBundle}, tonemapping::{DebandDither, Tonemapping}}, input::mouse::MouseWheel, math::{vec2, vec3}, prelude::*, render::camera::ScalingMode, window::PrimaryWindow};
use bevy_light_2d::light::AmbientLight2d;

use crate::core::{functions::ExpDecay, post_processing::PostProcessUniform};
pub struct EnhancedCameraPlugin;



impl Plugin for EnhancedCameraPlugin{
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_camera);
        app.add_systems(PostUpdate, update_camera);
    }
}


#[derive(Component)]
pub struct MainCamera;


#[derive(Component, Default)]
pub struct CameraFollow{
    pub speed: f32,
    pub order: u32
}


#[derive(Component, Default)]
pub struct CameraController{
    scale: f32,
    scale_translation_speed: Option<f32>,
    pub corners: Option<(Vec2, Vec2)>
}



impl CameraController{
    fn set_scale(&mut self, new_scale: f32, translation_speed: Option<f32>){
        self.scale = new_scale;
        self.scale_translation_speed = translation_speed;
    }
}

pub const TARGET_WIDTH : f32 = 400.;
pub const TARGET_HEIGHT : f32 = 300.;
pub const TARGET_ASPECT : f32 = TARGET_WIDTH / TARGET_HEIGHT;

fn setup_camera(
    mut commands: Commands,
){
    commands.spawn((
        Camera2dBundle{
            camera: Camera{
                hdr: true,
                ..default()
            },
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: TARGET_WIDTH,
                    min_height: TARGET_HEIGHT,
                },
                near: -800.,
                far: 800.,
                ..default()
            },
            tonemapping: Tonemapping::None,
            deband_dither: DebandDither::Disabled,
            transform: Transform::from_scale(Vec3::splat(1.)),
            ..default()
        },
        MainCamera,
        BloomSettings{
            intensity: 0.5,
            low_frequency_boost: 0.8,
            low_frequency_boost_curvature: 0.8,
            high_pass_frequency: 1.6,
            prefilter_settings: BloomPrefilterSettings{
                threshold: 0.6,
                threshold_softness: 0.7
            },
            composite_mode: BloomCompositeMode::Additive
        },
        AmbientLight2d {
            brightness: 1.,
            ..default()
        },
        CameraController{scale: 1., ..default()},
    ));
}

struct CameraScale(f32);

impl Default for CameraScale{
    fn default() -> Self {
        CameraScale(1.)
    }
}




fn update_camera(
    mut cameras_q: Query<(&mut Transform, &GlobalTransform, &mut CameraController, &Camera)>,
    targets_q: Query<(&Transform, &CameraFollow), (With<CameraFollow>, Without<CameraController>)>,
    time: Res<Time>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut camera_scale: Local<CameraScale>,
){
    let mut follow_position = Vec3::ZERO;
    let mut highest = 0;
    let mut follow_speed = 0.;
    for (tansform, follow) in targets_q.iter(){
        if highest <= follow.order {
            follow_position = tansform.translation;
            highest = follow.order;
            follow_speed = follow.speed;
        }
    }
    

    //for ev in evr_scroll.read() {
    //    camera_scale.0 -= ev.y * 0.1;
    //    camera_scale.0 = camera_scale.0.clamp(0.5, 3.);
    //}

    for (mut camera_transform, glob, mut controller, camera) in cameras_q.iter_mut(){
        controller.scale = camera_scale.0 * camera_scale.0;
        if let Some(corners) = controller.corners  {
            let right = corners.0.x.max(corners.1.x);
            let left = corners.0.x.min(corners.1.x);
            let up = corners.0.y.max(corners.1.y);
            let down = corners.0.y.min(corners.1.y);
            let world_screen_size = camera.logical_viewport_size().unwrap();
            let Some(a) = camera.viewport_to_world(&glob, vec2(0., 0.)).map(|ray| ray.origin.truncate()) else {continue;};
            let Some(b) = camera.viewport_to_world(&glob, world_screen_size).map(|ray| ray.origin.truncate()) else {continue;};
            let world_screen_size = (b - a) * vec2(0.5, -0.5);
            let size = (corners.0 - corners.1).abs();
            let center = (corners.0 + corners.1) / 2.;
            
            controller.scale_translation_speed = Some(10.);
            camera_transform.translation = camera_transform.translation.exp_decay(follow_position, follow_speed, time.delta_seconds());
            let Some(scale_speed) = controller.scale_translation_speed else {camera_transform.scale = Vec2::splat(controller.scale).extend(1.);continue};
            camera_transform.scale = Vec2::splat(camera_transform.scale.x.exp_decay(controller.scale, scale_speed, time.delta_seconds())).extend(1.);
            
            if camera_transform.translation.x > right - world_screen_size.x {
                camera_transform.translation.x = right - world_screen_size.x;
            }
            if camera_transform.translation.x < left + world_screen_size.x {
                camera_transform.translation.x = left + world_screen_size.x;
            }
            if camera_transform.translation.y > up - world_screen_size.y {
                camera_transform.translation.y = up - world_screen_size.y;

            }
            if camera_transform.translation.y < down + world_screen_size.y {
                camera_transform.translation.y = down + world_screen_size.y;
            }
            if world_screen_size.x * 2. > size.x{
                camera_transform.translation.x = center.x;
            }
            if world_screen_size.y * 2. > size.y{
                camera_transform.translation.y = center.y;
            }
        } else {
            controller.scale_translation_speed = Some(10.);
        camera_transform.translation = camera_transform.translation.exp_decay(follow_position, follow_speed, time.delta_seconds());
        let Some(scale_speed) = controller.scale_translation_speed else {camera_transform.scale = Vec2::splat(controller.scale).extend(1.);continue};
        camera_transform.scale = Vec2::splat(camera_transform.scale.x.exp_decay(controller.scale, scale_speed, time.delta_seconds())).extend(1.);
        }
        
    }
}

}
