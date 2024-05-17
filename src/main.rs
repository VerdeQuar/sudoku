use bevy::{
    ecs::removal_detection, prelude::*, render::render_resource::FilterMode,
    utils::tracing::instrument::WithSubscriber,
};
use bevy_defer::{
    async_system, async_systems::AsyncSystem, cancellation::Cancellation, signal_ids,
    signals::Sender, AsyncCommandsExtension, AsyncPlugin,
};
use bevy_ecs_tilemap::{
    helpers::{filling::fill_tilemap, geometry::get_tilemap_center_transform},
    map::{TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    prelude::{ArrayTextureLoader, TilemapArrayTexture},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    TilemapBundle, TilemapPlugin,
};
use bevy_inspector_egui::{egui::debug_text::print, quick::StateInspectorPlugin};
use rand::{thread_rng, Rng};
use std::fs::File;
use std::{rc::Rc, time::Instant};
use sudoku::Solution;

use crate::{
    exact_cover::SolvingState,
    sudoku::{Choice, Sudoku},
};
use rand::prelude::SliceRandom;
mod camera;
mod dancing_links;
mod exact_cover;
mod helpers;
mod sudoku;

fn main() {
    App::new()
        .add_plugins(AsyncPlugin::default_settings())
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TilemapPlugin,
        ))
        .add_systems(Startup, (setup, (generate_board).after(setup)))
        .add_systems(Update, camera::movement)

        .init_resource::<camera::CameraControl>()
            // .add_systems(Update, (show_solution))
        .run();
}

#[derive(Component)]
struct SudokuBoardFG;

#[derive(Component)]
struct SudokuBoardBG;

signal_ids! {
    OnSolutionFound: Solution,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    array_texture_loader: Res<ArrayTextureLoader>,
) {
    commands.spawn(Camera2dBundle::default());
    let board_size = TilemapSize::new(9, 9);

    let tile_size = TilemapTileSize::new(16. * 2., 16. * 2.);
    let grid_size = TilemapGridSize::new(16. * 2., 16. * 2.);
    let map_type = TilemapType::default();
    let texture_handle = asset_server.load("test_wfc.png");

    array_texture_loader.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(texture_handle.clone()),
        tile_size: TilemapTileSize::new(16., 16.),
        ..Default::default()
    });

    let board_fg_entity = commands.spawn_empty().id();
    let board_fg_id = TilemapId(board_fg_entity);
    let mut board_fg_tile_storage = TileStorage::empty(board_size);

    fill_tilemap(
        TileTextureIndex((26 * 4) + 10),
        board_size,
        board_fg_id,
        &mut commands,
        &mut board_fg_tile_storage,
    );

    commands
        .entity(board_fg_entity)
        .insert(TilemapBundle {
            grid_size,
            size: board_size,
            map_type,
            texture: TilemapTexture::Single(texture_handle.clone()),
            tile_size,
            storage: board_fg_tile_storage,
            transform: get_tilemap_center_transform(&board_size, &grid_size, &map_type, 0.),
            ..Default::default()
        })
        .insert(SudokuBoardFG);

    let board_bg_entity = commands.spawn_empty().id();
    let board_bg_id = TilemapId(board_bg_entity);
    let mut board_bg_tile_storage = TileStorage::empty(board_size);

    fill_tilemap(
        TileTextureIndex(27),
        board_size,
        board_bg_id,
        &mut commands,
        &mut board_bg_tile_storage,
    );

    commands
        .entity(board_bg_entity)
        .insert(TilemapBundle {
            grid_size,
            size: board_size,
            map_type,
            texture: TilemapTexture::Single(texture_handle.clone()),
            tile_size,
            storage: board_bg_tile_storage,
            transform: get_tilemap_center_transform(&board_size, &grid_size, &map_type, 1.),
            ..Default::default()
        })
        .insert(SudokuBoardBG);
}

fn generate_board(
    board_fg_query: Query<(&TilemapSize, &TileStorage), With<SudokuBoardFG>>,
    board_bg_query: Query<(&TilemapSize, &TileStorage), With<SudokuBoardBG>>,
    mut tile_query: Query<&mut TileTextureIndex>,
) {
    let mut n = 0;
    if let Ok((tilemap_size, _)) = board_fg_query.get_single() {
        n = (tilemap_size.x as f64).sqrt() as u32;
    }

    let mut filled_board: Solution = vec![];
    let mut removed_choices = vec![];
    let sudoku = Sudoku::new(n, vec![]);

    sudoku.solve(|solution| {
        filled_board = solution;
        return SolvingState::Abort;
    });

    for choice in filled_board.clone().into_iter() {
        removed_choices.push(choice);
        let now = Instant::now();

        let board = filled_board
            .iter()
            .copied()
            .filter(|c| !removed_choices.contains(c))
            .collect::<Vec<Choice>>();
        let sudoku = Sudoku::new(n, board);

        let mut solutions_found = 0;
        sudoku.solve(|_| {
            solutions_found += 1;

            if solutions_found <= 1 {
                return SolvingState::Continue;
            } else {
                return SolvingState::Abort;
            }
        });
        println!("{}", now.elapsed().as_millis());
        println!("found {} solutions", solutions_found);

        if solutions_found > 1 {
            removed_choices.pop();
        }
    }

    let mut solution = filled_board.clone();

    for choice in removed_choices {
        solution.remove(solution.iter().position(|c| *c == choice).unwrap());
    }

    if let Ok((_, tile_storage)) = board_fg_query.get_single() {
        for choice in solution {
            let pos = TilePos {
                x: choice.row as u32,
                y: choice.column as u32,
            };
            let texture_offset = if n <= 3 { 26 * 4 } else { 26 * 3 };
            if let Some(tile_entity) = tile_storage.get(&pos) {
                if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
                    tile.0 = texture_offset + (choice.number as u32);
                }
            }
        }
    }

    if let Ok((tilemap_size, tile_storage)) = board_bg_query.get_single() {
        let n = (tilemap_size.x as f64).sqrt() as u32;
        for x in 0..tilemap_size.x {
            for y in 0..tilemap_size.y {
                let pos = TilePos { x, y };
                if let Some(tile_entity) = tile_storage.get(&pos) {
                    if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
                        let x_offset = match x % n {
                            0 => 0,
                            nx if nx == n - 1 => 2,
                            _ => 1,
                        };
                        let y_offset = match y % n {
                            0 => 0,
                            ny if ny == n - 1 => 2,
                            _ => 1,
                        };

                        tile.0 = 2 * 26 + x_offset - (y_offset * 26);
                    }
                }
            }
        }
    }
}

// fn show_solution() {
//     let mut now = Instant::now();

// for event in reader.read() {
//     let elapsed_time = now.elapsed();
//     println!("took {}us", elapsed_time.as_micros());
//     now = Instant::now();
// }
// solver_handle.0.abort();

// if let Ok(tile_storage) = tilemap_query.get_single() {
//     for x in 0..9 {
//         for y in 0..9 {
//             if let Some(tile_entity) = tile_storage.get(&TilePos { x, y }) {
//                 if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
//                     tile.0 = solver.board[x as usize][y as usize]
//                 }
//             }
//         }
//     }
// }
// }
// const show_solution: AsyncSystem = async_system!(|recv: Receiver<OnSolutionFound>| {
//     let solution: Solution = recv.recv().await;
// });
