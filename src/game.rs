use std::cmp;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use crate::mine_core::{ BlockType, BlockStatus, MinePlayground, MineBlock, Position, ClickResult };

pub fn game_app(config: GameConfig) {
    App::build()
        .add_resource(WindowDescriptor {
            vsync: false,
            width: cmp::max(config.width * BLOCK_WIDTH, MIN_WIDTH) as f32,
            height: cmp::max(config.height * BLOCK_WIDTH + Y_MARGIN, MIN_HEIGHT) as f32,
            title: String::from("Mine Sweeper"),
            resizable: false,
            ..Default::default()
        })
        .add_resource(config)
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_resource(CursorLocation(Vec2::new(0.0, 0.0)))
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_resource(State::new(GameState::Prepare))
            .add_startup_system(setup.system())
            .add_system(fps_update.system())
            .add_system(debug_text_update.system())
            .add_system(restart_button_system.system())
            .add_startup_system(new_map.system())
            .add_system(handle_movement.system())
            .add_system(handle_click.system())
            .add_system(render_map.system())
            .add_stage_after(stage::UPDATE, STAGE, StateStage::<GameState>::default())
            .on_state_enter(STAGE, GameState::Prepare, init_map_render.system())
            .on_state_enter(STAGE, GameState::Ready, new_map.system());
    }
}

const BLOCK_WIDTH: usize = 24;
const MIN_HEIGHT: usize = 160;
const MIN_WIDTH: usize = 160;
const Y_MARGIN: usize = 50;
const SPRITE_SIZE: f32 = 48.0;
const STAGE: &str = "game_state";
const NEW_GAME_TEXT: &str = "New Game";

struct RefreshButton;
struct DebugText;
struct MapData {
    map_entity: Entity,
}

struct WindowOffset {
    x: f32,
    y: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct GameConfig {
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    Prepare,
    Ready,
    Running,
    Over,
}

#[derive(Default, Debug)]
struct CursorLocation(Vec2);
struct LastActionText(String);

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}
impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}
impl MineBlock {
    fn get_sprite_index(&self) -> usize {
        match self.bstatus {
            BlockStatus::Flaged => 3,
            BlockStatus::Shown => {
                match self.btype {
                    BlockType::Mine => 1,
                    _ => 0,
                }
            },
            _ => 2,
        }
    }
}

struct FpsRefresh;

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    windows: ResMut<Windows>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let font = asset_server.load("fonts/pointfree.ttf");
    let window = windows.get_primary().unwrap();
    commands
        .spawn(CameraUiBundle::default())
        .spawn(Camera2dBundle::default());
    commands.spawn(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text {
            value: "debug text here".to_string(),
            font: font.clone(),
            style: TextStyle {
                font_size: 18.0,
                color: Color::rgba(0.0, 0.5, 0.5, 0.5),
                alignment: TextAlignment::default(),
            },
        },
        ..Default::default()
    }).with(DebugText);
    commands
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                value: "-".to_string(),
                font: font.clone(),
                style: TextStyle {
                    font_size: 20.0,
                    color: Color::rgba(0.0, 0.5, 0.5, 0.5),
                    alignment: TextAlignment::default(),
                },
            },
            ..Default::default()
        })
        .with(FpsRefresh);
    commands.insert_resource(WindowOffset {
        x: window.physical_width() as f32 / 2.0 - BLOCK_WIDTH as f32 / 2.0,
        y: window.physical_height() as f32 / 2.0 - BLOCK_WIDTH as f32 / 2.0,
    });
    commands
        .insert_resource(LastActionText(NEW_GAME_TEXT.to_string()))
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(25.0)),
                position_type: PositionType::Absolute,
                // center button
                position: Rect {
                    left: Val::Px((window.physical_width() as f32) / 2.0 - 50.0),
                    top: Val::Px(5.0),
                    ..Default::default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: "New Game".to_string(),
                    font: asset_server.load("fonts/pointfree.ttf"),
                    style: TextStyle {
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            }).with(RefreshButton);
        });

    let texture_handle = asset_server.load("textures/block.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(SPRITE_SIZE, SPRITE_SIZE), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(texture_atlas_handle);
}
struct RenderBlock {
    pos: Position,
}
fn new_map(
    commands: &mut Commands,
    config: Res<GameConfig>,
) {
    commands
        .insert_resource(MinePlayground::init(&config.width, &config.height, &config.mine_count).unwrap());
    commands.spawn((MinePlayground::init(&config.width, &config.height, &config.mine_count).unwrap(), ));
    commands.insert_resource(MapData {
        map_entity: commands.current_entity().unwrap(),
    });
}

fn init_map_render(
    commands: &mut Commands,
    texture_atlases: Res<Assets<TextureAtlas>>,
    atlas_handle: Res<Handle<TextureAtlas>>,
    window_offset: Res<WindowOffset>,
    config: Res<GameConfig>,
    mut game_state: ResMut<State<GameState>>,
) {
    println!("111init_map_render run once");
    for y in 0..config.height {
        for x in 0..config.width {
            let texture_atlas = texture_atlases.get_handle(atlas_handle.clone());
            commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            (x * BLOCK_WIDTH) as f32 - window_offset.x,
                            (y * BLOCK_WIDTH) as f32 - window_offset.y,
                            0.0
                        ),
                        scale: Vec3::splat(0.5),
                        ..Default::default()
                    },
                    texture_atlas,
                    // TODO: rename
                    sprite: TextureAtlasSprite::new(2),
                    ..Default::default()
                })
                .with(RenderBlock { pos: Position { x, y } });
        }
    }
    println!("{:?}", game_state.current());
    game_state.set_next(GameState::Ready).unwrap();
}
fn render_map (
    query: Query<
        // components
        &MinePlayground,
        // filters
        Changed<MinePlayground>, 
    >,
    mut sprites: Query<(&mut TextureAtlasSprite, &RenderBlock)>,
) {
    for mp in query.iter() {
        println!("detect mp changed{:?}", mp.shown_count);
        for (mut sprite, rb) in sprites.iter_mut() {
            sprite.index = mp.map[rb.pos.y][rb.pos.x].get_sprite_index() as u32;
            // println!("x:{:?}-y:{:?}-block:{:?}-index:{:?}", rb.pos.x, rb.pos.y, mp.map[rb.pos.y][rb.pos.x], mp.map[rb.pos.y][rb.pos.x].get_sprite_index() as u32);
        }
    }
}
fn handle_movement(
    mut cursor_pos: ResMut<CursorLocation>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut evr_cursor: Local<EventReader<CursorMoved>>,
) {
    for ev in evr_cursor.iter(&cursor_moved_events) {
        cursor_pos.0 = ev.position;
    }
}

fn handle_click(
    btns: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorLocation>,
    config: Res<GameConfig>,
    mut mquery: Query<&mut MinePlayground>,
    map_data: Res<MapData>,
    mut text_query: Query<&mut Text, With<RefreshButton>>,
    mut last_action_text: ResMut<LastActionText>,
    mut game_state: ResMut<State<GameState>>,
) {
    if btns.just_released(MouseButton::Left) {
        if let Some((x, y)) = get_block_index_by_cursor_pos(cursor_pos.0, *config) {
            println!("{:?}-{:?}", x, y);
            let mut mp: Mut<MinePlayground> = mquery.get_component_mut(map_data.map_entity).unwrap();
            let click_result = mp.click(&x, &y);
            println!("{:?}", click_result);
            if let ClickResult::Wasted = click_result {
                let mut text = text_query.iter_mut().next().unwrap();
                text.value = String::from("Game Over");
                *last_action_text = LastActionText(String::from("Game Over"));
                game_state.set_next(GameState::Over).unwrap();
            } else if let GameState::Ready = game_state.current()  {
                game_state.set_next(GameState::Running).unwrap();
            }
        }
    }
    if btns.just_released(MouseButton::Right) {
        if let Some((x, y)) = get_block_index_by_cursor_pos(cursor_pos.0, *config) {
            println!("{:?}-{:?}", x, y);
            if let GameState::Ready = game_state.current()  {
                game_state.set_next(GameState::Running).unwrap();
            }
            let mut mp: Mut<MinePlayground> = mquery.get_component_mut(map_data.map_entity).unwrap();
            mp.right_click(&x, &y);
        }
    }
}

fn fps_update(
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<FpsRefresh>>,
) {
    for mut text in query.iter_mut() {
        let mut fps = 0.0;
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_avg) = fps_diagnostic.average() {
                fps = fps_avg;
            }
        }

        text.value = format!(
            "{:.1} fps",
            fps,
        );
    }
}
fn debug_text_update(
    mut query: Query<&mut Text, With<DebugText>>,
    game_state: Res<State<GameState>>,
) {
    for mut text in query.iter_mut() {
        text.value = format!("state: {:?}", game_state.current());
    }
}

fn restart_button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Mutated<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut last_action_text: ResMut<LastActionText>,
    mut game_state: ResMut<State<GameState>>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
                text.value = NEW_GAME_TEXT.to_string();
                *last_action_text = LastActionText(NEW_GAME_TEXT.to_string());
                if *game_state.current() != GameState::Prepare {
                    game_state.set_next(GameState::Prepare).unwrap();
                }
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
                text.value = NEW_GAME_TEXT.to_string();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
                text.value = (*last_action_text.0).to_string();
            }
        }
    }
}

fn get_block_index_by_cursor_pos(pos: Vec2, config: GameConfig) -> Option<(usize, usize)> {
    let x = (pos.x / BLOCK_WIDTH as f32).floor() as usize;
    let y = (pos.y / BLOCK_WIDTH as f32).floor() as usize;
    if (0..config.height).contains(&y) && (0..config.width).contains(&x) {
        return Some((x, y));
    }
    None
}
