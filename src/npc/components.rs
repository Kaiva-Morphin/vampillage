use bevy::{prelude::*, time::Stopwatch};

#[derive(Component)]
pub struct Civilian;

#[derive(Component)]
pub struct Hunter;

#[derive(Component, Debug)]
pub struct Projectile;

#[derive(Component)]
pub struct DespawnTimer {
    pub timer: Timer
}

#[derive(Component)]
pub struct HunterTimer {
    pub timer: Timer
}

#[derive(Component)]
pub struct NpcVelAccum {
    pub v: Vec2,
}

#[derive(Component)]
pub struct NpcPath {
    pub path: Option<Vec<IVec2>>,
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum NpcState {
    Attack,
    Escape,
    Chase,
    Look,
    Chill,
    Dead,
}

#[derive(Component)]
pub struct ChillTimer {
    pub timer: Timer
}

#[derive(Component)]
pub struct PlayerLastPos {
    pub pos: IVec2,
}

#[derive(Component)]
pub struct AttackTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct ParticleTimer {
    pub timer: Timer,
}