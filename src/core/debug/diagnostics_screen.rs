pub mod plugin {

use bevy::{diagnostic::{DiagnosticPath, DiagnosticsStore}, prelude::*, utils::HashMap};

pub struct ScreenDiagnosticsPlugin;

impl Plugin for ScreenDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup);
        app.add_systems(Update, update);
    }
}


#[derive(Resource)]
pub struct ScreenDiagnostics{
    lines: HashMap<&'static str, ScreenDiagnosticsLine>,
    to_remove: Vec<Entity>,
    timer: Timer,
    changed_lines: Vec<&'static str>,
    font: Handle<Font>,
    enabled: bool,
    visible: bool,
    font_size: f32,
    root: Entity,
    layout_nodes: HashMap<ScreenDiagnosticsLineLayout, Entity>,
    bg_color: Color,
    font_color: Color
}

impl Default for ScreenDiagnostics {
    fn default() -> Self {
        ScreenDiagnostics {
            lines: HashMap::new(),
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            to_remove: Vec::new(),
            changed_lines: Vec::new(),
            font: Handle::default(),
            enabled: false,
            visible: false,
            root: Entity::from_raw(0),
            layout_nodes: HashMap::new(),
            font_size: 24.,
            bg_color: Color::Srgba(Srgba{ red: 0., green: 0., blue: 0., alpha: 0.8 }),
            font_color: Color::Srgba(Srgba{ red: 1., green: 1., blue: 1., alpha: 1. }),
        }
    }
}

impl ScreenDiagnostics {
    pub fn show(&mut self){
        self.enabled = true;
    }
    pub fn switch(&mut self){
        self.enabled = !self.enabled;
    }
    pub fn hide(&mut self){
        self.enabled = false;
    }
    pub fn set_font(&mut self, font: Handle<Font>){
        self.font = font
    }

    pub fn set_line(&mut self, path: &'static str, line: DiagnosticsLine, layout: ScreenDiagnosticsLineLayout){
        if let Some(d_line) = self.lines.get_mut(path){
            d_line.line = line;
            self.changed_lines.push(path);
        } else {
            self.lines.insert(path, ScreenDiagnosticsLine{line: line, entity: None, layout});
            self.changed_lines.push(path);
        }
    }

    pub fn set_line_lu(&mut self, path: &'static str, line: DiagnosticsLine){
            if let Some(d_line) = self.lines.get_mut(path){
                d_line.line = line;
                self.changed_lines.push(path);
            } else {
                self.lines.insert(path, ScreenDiagnosticsLine{line: line, entity: None, layout: ScreenDiagnosticsLineLayout::LeftUp});
                self.changed_lines.push(path);
            }
        }
    pub fn set_line_ru(&mut self, path: &'static str, line: DiagnosticsLine){
            if let Some(d_line) = self.lines.get_mut(path){
                d_line.line = line;
                self.changed_lines.push(path);
            } else {
                self.lines.insert(path, ScreenDiagnosticsLine{line: line, entity: None, layout: ScreenDiagnosticsLineLayout::RightUp});
                self.changed_lines.push(path);
            }
        }
    pub fn set_line_ld(&mut self, path: &'static str, line: DiagnosticsLine){
            if let Some(d_line) = self.lines.get_mut(path){
                d_line.line = line;
                self.changed_lines.push(path);
            } else {
                self.lines.insert(path, ScreenDiagnosticsLine{line: line, entity: None, layout: ScreenDiagnosticsLineLayout::LeftDown});
                self.changed_lines.push(path);
            }
        }
    pub fn set_line_rd(&mut self, path: &'static str, line: DiagnosticsLine){
        if let Some(d_line) = self.lines.get_mut(path){
            d_line.line = line;
            self.changed_lines.push(path);
        } else {
            self.lines.insert(path, ScreenDiagnosticsLine{line: line, entity: None, layout: ScreenDiagnosticsLineLayout::RightDown});
            self.changed_lines.push(path);
        }
    }
    
    pub fn update_line(&mut self, path: &'static str, line: DiagnosticsLine){
        if let Some(d_line) = self.lines.get_mut(path){
            d_line.line = line;
            self.changed_lines.push(path);
    } else {
            warn!("Attemt to update non-existing line: {path}");
        }
    }

    /*pub fn forced_update_all(&mut self, commands: &mut Commands){

    }*/

    pub fn update(&mut self, commands: &mut Commands){
        for e in self.to_remove.iter(){
            commands.entity(*e).despawn();
        }
        for path in self.changed_lines.iter(){
            if let Some(line) = self.lines.get_mut(*path){
                if let Some(e) = line.entity{
                    commands.entity(e).insert((
                        Text::from_sections([
                            TextSection{
                                value: line.line.prefix.clone(),
                                style: TextStyle {
                                    font: self.font.clone(),
                                    font_size: self.font_size,
                                    color: line.line.prefix_color.unwrap_or(self.font_color),
                                },
                            },
                            TextSection{
                                value: line.line.text.clone(),
                                style: TextStyle {
                                    font: self.font.clone(),
                                    font_size: self.font_size,
                                    color: line.line.text_color.unwrap_or(self.font_color),
                                },
                            },
                            TextSection{
                                value: line.line.postfix.clone(),
                                style: TextStyle {
                                    font: self.font.clone(),
                                    font_size: self.font_size,
                                    color: line.line.postfix_color.unwrap_or(self.font_color),
                                },
                            }]
                        ),
                        Name::new(format!("path: {path}"))
                    ));
                } else if let Some(parent) = self.layout_nodes.get(&line.layout){
                        line.entity = Some(
                            commands.spawn((
                                TextBundle::from_sections([
                                    TextSection{
                                        value: line.line.prefix.clone(),
                                        style: TextStyle {
                                            font: self.font.clone(),
                                            font_size: self.font_size,
                                            color: line.line.prefix_color.unwrap_or(self.font_color),
                                        },
                                    },
                                    TextSection{
                                        value: line.line.text.clone(),
                                        style: TextStyle {
                                            font: self.font.clone(),
                                            font_size: self.font_size,
                                            color: line.line.text_color.unwrap_or(self.font_color),
                                        },
                                    },
                                    TextSection{
                                        value: line.line.postfix.clone(),
                                        style: TextStyle {
                                            font: self.font.clone(),
                                            font_size: self.font_size,
                                            color: line.line.postfix_color.unwrap_or(self.font_color),
                                        },
                                }]).with_background_color(self.bg_color), 
                            Name::new(format!("path: {path}"))
                            )
                        ).set_parent(*parent).id());
                    } else {
                        warn!("Cant find parent node for {:?}", line.layout);
                    }
            } else {
                warn!("Attemt to update non-existing line: {path}");
            }
        }

        self.changed_lines.clear();

        if self.visible && !self.enabled{ // hide
            commands.entity(self.root).insert(Visibility::Hidden);
            self.visible = false;
        } else
        if !self.visible && self.enabled{ // show
            commands.entity(self.root).insert(Visibility::Inherited);
            self.visible = true;
        }
    }

    pub fn add_line(&mut self, path: &'static str, line: DiagnosticsLine, layout: ScreenDiagnosticsLineLayout){
        self.lines.insert(path, ScreenDiagnosticsLine{line: line, entity: None, layout});
        self.changed_lines.push(path);
    }
    pub fn remove_line(&mut self, path: &'static str){
        if let Some(value) = self.lines.remove(path){
            if let Some(entity) = value.entity{
                self.to_remove.push(entity);
                return;
            }
        }
        warn!("Attemt to remove non-existing line: {path}");
    }
}

#[derive(Component, Debug, Eq, PartialEq, Default, Hash, Clone, Copy)]
#[repr(u8)]
pub enum ScreenDiagnosticsLineLayout{
    #[default]
    LeftUp,
    RightUp,
    LeftDown,
    RightDown
}

impl ScreenDiagnosticsLineLayout {

}


#[derive(Component)]
struct ScreenDiagnosticsRoot;

#[derive(Default)]
pub struct DiagnosticsLine{
    pub prefix: String,
    pub prefix_color: Option<Color>,
    pub text: String,
    pub text_color: Option<Color>,
    pub postfix: String,
    pub postfix_color: Option<Color>,
}

impl From<String> for DiagnosticsLine {
    fn from(value: String) -> Self {
        DiagnosticsLine::new(value)
    }
}

impl From<&'static str> for DiagnosticsLine {
    fn from(value: &'static str) -> Self {
        DiagnosticsLine::new(value.to_string())
    }
}



impl DiagnosticsLine{
    pub fn new(text: String) -> Self{
        DiagnosticsLine{text,..default()}
    }
    pub fn with_text_color(mut self, color: Color) -> Self{
        self.text_color = Some(color);
        self
    }
    pub fn with_prefix_color(mut self, color: Color) -> Self{
        self.prefix_color = Some(color);
        self
    }
    pub fn with_postfix_color(mut self, color: Color) -> Self{
        self.postfix_color = Some(color);
        self
    }
    pub fn with_prefix(mut self, text: String) -> Self{
        self.prefix = text;
        self
    }
    pub fn with_postfix(mut self, text: String) -> Self{
        self.postfix = text;
        self
    }
    pub fn with_text(mut self, text: String) -> Self{
        self.text = text;
        self
    }
}

#[derive(Default)]
struct ScreenDiagnosticsLine{
    pub line: DiagnosticsLine,
    pub entity: Option<Entity>,
    pub layout: ScreenDiagnosticsLineLayout,
}

impl ScreenDiagnosticsLine{

}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>
){
    let font = asset_server.load("fonts/Monocraft.ttf");

    let root_node = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        },
        Name::new("DebugRoot"),
        ScreenDiagnosticsRoot
    )).id();

    let lu = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                align_content: AlignContent::Start,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },
        Name::new("LeftUp"),
        ScreenDiagnosticsLineLayout::LeftUp
    )).set_parent(root_node).id();

    let ld = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::End,
                align_content: AlignContent::Start,
                align_items: AlignItems::Start,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        Name::new("LeftDown"),
        ScreenDiagnosticsLineLayout::LeftDown
    )).set_parent(root_node).id();

    let ru = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::End,
                align_content: AlignContent::Start,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        Name::new("RightUp"),
        ScreenDiagnosticsLineLayout::RightUp
    )).set_parent(root_node).id();

    let rd = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::End,
                align_items: AlignItems::End,
                align_content: AlignContent::End,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        Name::new("RightDown"),
        ScreenDiagnosticsLineLayout::RightDown
    )).set_parent(root_node).id();
    let mut layout_nodes = HashMap::new();
    layout_nodes.insert(ScreenDiagnosticsLineLayout::RightUp, ru);
    layout_nodes.insert(ScreenDiagnosticsLineLayout::RightDown, rd);
    layout_nodes.insert(ScreenDiagnosticsLineLayout::LeftUp, lu);
    layout_nodes.insert(ScreenDiagnosticsLineLayout::LeftDown, ld);
    let mut diagnostics = ScreenDiagnostics{
        layout_nodes,
        root: root_node,
        font: font.clone(),
        ..default()
    };
    diagnostics.show();
    
    diagnostics.add_line(
        "PKGINFO", 
        DiagnosticsLine::new(format!(" v{VERSION}")).with_text_color(Color::Srgba(Srgba::gray(0.5))).with_prefix_color(Color::Srgba(Srgba::gray(0.5))).with_prefix(NAME.to_owned()), 
        ScreenDiagnosticsLineLayout::RightDown
    );
    diagnostics.add_line(
        "fps_avg", 
        DiagnosticsLine::new("?".to_string()).with_postfix(" avg fps".to_owned()).with_postfix_color(Color::Srgba(Srgba::gray(0.5))),
        ScreenDiagnosticsLineLayout::RightDown
    );
    diagnostics.add_line(
        "fps", 
        DiagnosticsLine::new("?".to_string()).with_postfix(" fps".to_owned()).with_postfix_color(Color::Srgba(Srgba::gray(0.5))), 
        ScreenDiagnosticsLineLayout::RightDown
    );
    commands.insert_resource(diagnostics);
    info!("Debugger plugin inited!");
}

fn update(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut diagnostics: ResMut<ScreenDiagnostics>,
    diagnostics_store: Option<Res<DiagnosticsStore>>,
    time: Res<Time<Real>>,
    mut commands: Commands
){
    if keyboard.just_pressed(KeyCode::F3){
        diagnostics.switch();
    }

    if diagnostics_store.is_none() {
        error_once!("Cant find DiagnosticsStore! make shure ure are add FrameTimeDiagnosticsPlugin!");
        return;
    }
    let diagnostics_store = diagnostics_store.expect("Err");
    if diagnostics.timer.tick(time.delta()).finished(){
        let fps = diagnostics_store.get(&DiagnosticPath::const_new("fps"));
        if let Some(fps) = fps {
            let smoothed = fps.smoothed().unwrap_or(0.);
            let avg = fps.average().unwrap_or(0.);

            let get_fps_color = |x: f64| -> Color {
                if x > 59. {
                    Color::srgba(0., 1., 0., 1.)
                } else if x > 49. {
                    Color::srgba(0.2, 0.8, 0., 1.)
                } else if x > 39. {
                    Color::srgba(0.4, 0.6, 0., 1.)
                } else if x > 10. {
                    Color::srgba(0.6, 0.4, 0., 1.)
                } else if x > 5. {
                    Color::srgba(0.8, 0.2, 0., 1.)
                } else {
                    Color::srgba(0.1, 0., 0., 1.)
                }
            };
            diagnostics.update_line(
                "fps", 
                DiagnosticsLine::new(format!("{:.0}", smoothed)).with_text_color(get_fps_color(smoothed)).with_postfix(" fps".to_owned()).with_postfix_color(Color::Srgba(Srgba::gray(0.5))), 
            );  
            diagnostics.update_line(
                "fps_avg", 
                DiagnosticsLine::new(format!("{:.0}", avg)).with_text_color(get_fps_color(avg)).with_postfix(" fps avg".to_owned()).with_postfix_color(Color::Srgba(Srgba::gray(0.5))), 
            );  
        }
    }
    
    diagnostics.update(&mut commands);
}

}