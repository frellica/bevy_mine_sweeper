use std::cmp;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use crate::mine_core::{ BlockType, BlockStatus, MinePlayground, MineBlock };

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
        .init_resource::<ButtonMaterials>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_resource(State::new(GameState::Ready))
        .add_startup_system(setup.system())
        .add_system(fps_update.system())
        .add_system(restart_button_system.system())
        .add_startup_system(new_map.system())
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<GameState>::default())
        .on_state_enter(STAGE, GameState::Ready, new_game.system())
        .run();
}

const BLOCK_WIDTH: usize = 24;
const MIN_HEIGHT: usize = 160;
const MIN_WIDTH: usize = 160;
const Y_MARGIN: usize = 50;
const SPRITE_SIZE: f32 = 48.0;
const STAGE: &str = "game_state";

struct WindowOffset {
    x: f32,
    y: f32,
}

pub struct GameConfig {
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
}

#[derive(Debug, Clone)]
enum GameState {
    Ready,
    Running,
    Over,
}

// TODO: use texture?
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
            _ => 2,
            BlockStatus::Flaged => 3,
            BlockStatus::Shown => {
                match self.btype {
                    BlockType::Mine => 1,
                    _ => 0,
                }
            },
        }
    }
}

struct FpsRefresh;

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    windows: ResMut<Windows>,
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
    });
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
                    value: "Restart".to_string(),
                    font: asset_server.load("fonts/pointfree.ttf"),
                    style: TextStyle {
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        });
}
struct RenderBlock;
fn new_map(
    commands: &mut Commands,
    config: Res<GameConfig>,
) {
    commands
        .insert_resource(MinePlayground::init(&config.width, &config.height, &config.mine_count).unwrap());
}

fn new_game(
    commands: &mut Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mp: Res<MinePlayground>,
    window_offset: Res<WindowOffset>,
) {
    let texture_handle = asset_server.load("textures/block.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(SPRITE_SIZE, SPRITE_SIZE), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    println!("111{:?}", mp.map);
    for row in mp.map.iter() {
        for block in row.iter() {
            let texture_atlas = texture_atlases.get_handle(texture_atlas_handle.clone());
            // println!("atlas:{:?}", texture_atlas);
            
            commands
                .spawn((RenderBlock, ))
                .with_bundle(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            (block.pos.x * BLOCK_WIDTH) as f32 - window_offset.x,
                            (block.pos.y * BLOCK_WIDTH) as f32 - window_offset.y,
                            0.0
                        ),
                        scale: Vec3::splat(0.5),
                        ..Default::default()
                    },
                    texture_atlas,
                    sprite: TextureAtlasSprite::new(block.get_sprite_index() as u32),
                    ..Default::default()
                });
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

fn restart_button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Mutated<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        // let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}