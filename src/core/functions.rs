use bevy::{prelude::*, utils::HashMap};

pub trait ExpDecay<T> {
    fn exp_decay(&self, b: T, decay: f32, dt: f32) -> T;
}

impl ExpDecay<f32> for f32 {
    fn exp_decay(&self, b: f32, decay: f32, dt: f32) -> f32 {
        b + (self - b) * (-decay*dt).exp()
    }
}

impl ExpDecay<Vec3> for Vec3 {
    fn exp_decay(&self, b: Vec3, decay: f32, dt: f32) -> Vec3 {
        b + (*self - b) * (-decay*dt).exp()
    }
}

impl ExpDecay<Vec2> for Vec2 {
    fn exp_decay(&self, b: Vec2, decay: f32, dt: f32) -> Vec2 {
        b + (*self - b) * (-decay*dt).exp()
    }
}

#[derive(Resource, Default)]
pub struct TextureAtlasLayoutHandles{
    handles: HashMap<String, Handle<TextureAtlasLayout>>
}

impl TextureAtlasLayoutHandles {
    pub fn add_or_load(&mut self, asset_server: &Res<AssetServer>, name: &'static str, layout: TextureAtlasLayout) -> Handle<TextureAtlasLayout>{
        if let Some(atlas) = self.handles.get(&name.to_string()) {
            atlas.clone()
        } else {
            let handle = asset_server.add(layout);
            self.handles.insert(name.to_string(), handle.clone());
            handle
        }
    }
}