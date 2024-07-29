use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub hp: f32,
    pub xp: f32,
    pub score: f32,
    pub max_speed: f32,
    pub accumulation_gain: f32,
    pub dash_cd: f32,
    pub dash_tick: f32,
    pub phys_res: f32,
    pub hp_gain: f32,
    pub xp_gain: f32,
    pub hunger_rate: f32,
    pub max_xp: f32,
    pub max_hp: f32,
    pub is_dead: bool,
} 

impl Default for Player {
    fn default() -> Self {
        Player {
            hp: 80.,
            xp: 0., 
            score: 0., 
            max_speed: 60., 
            accumulation_gain: 500., 
            phys_res: 0.2, 
            dash_cd: 1.5,
            dash_tick: 1.,
            hp_gain: 5., 
            xp_gain: 10.,
            hunger_rate: 2.,
            max_xp: 100., 
            max_hp: 80., 
            is_dead: false
        }
    }
}


// armor (phys res); speed; hp gain; xp gain; max hp;

#[derive(Component)]
pub struct DashTimer {
    pub timer: Timer,
}

#[derive(Event)]
pub struct HitPlayer {
    pub dmg_type: u8,
}

#[derive(Event)]
pub struct KillNpc {
    pub npc_type: u8,
}

#[derive(Component)]
pub enum UpgradeButton {
    MaxHp,
    Armor,
    HpGain,
    XpGain,
    Speed,
    HungerRate,
    DashCD,
    DashTick
}

#[derive(Component)]
pub struct ParentEntity {
    pub entity: Entity
}

#[derive(Event)]
pub struct KillPlayer {
    pub won: bool,
}

#[derive(Resource)]
pub struct DeathTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct DeathTime;

#[derive(Component)]
pub struct DeathText;