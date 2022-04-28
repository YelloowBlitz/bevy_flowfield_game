use super::physics::collider::Collider;
use super::physics::rigidbody::Rigidbody;
use super::physics::trigger::Trigger;
use crate::player::Player;
use bevy::prelude::*;
use ncollide2d::na::{Isometry2, Vector2};
use ncollide2d::shape::Cuboid;
use rand::Rng;
use std::collections::HashSet;
use crate::Wall;

pub const FLOW_FIELD_LAYER: u32 = 0;

const BLOCK_SIZE_WIDTH: usize = 5; // in pixels
const BLOCK_SIZE_HEIGHT: usize = 5; // in pixels

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

impl GridPos {
    pub fn distance(&self, other: &Self) -> f32 {
        Vec2::new(self.x as f32, self.y as f32).distance(Vec2::new(other.x as f32, other.y as f32))
    }
}

impl From<(usize, usize)> for GridPos {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
pub struct Flowing;

pub struct FlowField {
    window_width: usize,
    window_height: usize,
    pub size_x: usize,
    pub size_y: usize,
    heat_grid: Vec<Vec<f32>>,
    pub flow_grid: Vec<Vec<Vec2>>,
}

impl FlowField {
    fn new(window_width: usize, window_height: usize) -> Self {
        let size_x = window_width.div_ceil(BLOCK_SIZE_WIDTH);
        let size_y = window_height.div_ceil(BLOCK_SIZE_HEIGHT);

        let flow_grid = vec![vec![Vec2::new(0., 0.); size_y]; size_x];
        let heat_grid = vec![vec![-1.; size_y]; size_x];

        Self {
            window_width,
            window_height,
            size_x,
            size_y,
            heat_grid,
            flow_grid,
        }
    }

    fn reset(&mut self, window_width: usize, window_height: usize) {
        self.window_width = window_width;
        self.window_height = window_height;

        self.size_x = window_width.div_ceil(BLOCK_SIZE_WIDTH);
        self.size_y = window_height.div_ceil(BLOCK_SIZE_HEIGHT);

        self.flow_grid = vec![vec![Vec2::new(0., 0.); self.size_y]; self.size_x];
        self.heat_grid = vec![vec![-1.; self.size_y]; self.size_x];
    }

    /// positions: Grid positions where the flowfield will flow into (the end points destination)
    fn build<T: Into<GridPos>>(
        &mut self,
        positions: HashSet<T>,
        colliders: Query<(&Collider, &Transform), With<Wall>>,
    ) {
        self.flow_grid = vec![vec![Vec2::new(0., 0.); self.size_y]; self.size_x];
        self.heat_grid = vec![vec![-1.; self.size_y]; self.size_x];

        let mut walls: HashSet<GridPos> = HashSet::new();

        for (col, trans) in colliders.iter() {
            if col.layer == FLOW_FIELD_LAYER {
                for x in 0..self.size_x {
                    for y in 0..self.size_y {
                        let block = Collider::new(
                            Box::new(Cuboid {
                                half_extents: Vector2::new(
                                    BLOCK_SIZE_WIDTH as f32 / 2.,
                                    BLOCK_SIZE_HEIGHT as f32 / 2.,
                                ),
                            }),
                            0,
                        );
                        let block_trans = Transform {
                            translation: Vec3::new(
                                (x * BLOCK_SIZE_WIDTH) as f32,
                                (y * BLOCK_SIZE_HEIGHT) as f32,
                                0.,
                            ),
                            ..Default::default()
                        };
                        if block.overlap(trans, col, &block_trans) {
                            walls.insert((x, y).into());
                        }
                    }
                }
            }
        }

        let mut queue = std::collections::vec_deque::VecDeque::new();
        let mut visited: HashSet<GridPos> = HashSet::with_capacity(self.size_x * self.size_y);

        for pos in positions.into_iter() {
            let pos_grid: GridPos = pos.into();
            self.heat_grid[pos_grid.x][pos_grid.y] = 0.;
            queue.push_back(pos_grid.clone());
            visited.insert(pos_grid);
        }

        // Calcul the heat grid
        while !queue.is_empty() {
            let pos_grid = queue.pop_front().unwrap();

            let neighbors: HashSet<GridPos> = self
                .neighbors(&pos_grid)
                .into_iter()
                .filter(|p| !walls.contains(p))
                .collect();

            for neighbor in neighbors.into_iter() {
                let d = neighbor.distance(&pos_grid);

                if visited.insert(neighbor.clone()) {
                    self.heat_grid[neighbor.x][neighbor.y] =
                        self.heat_grid[pos_grid.x][pos_grid.y] + d;
                    queue.push_back(neighbor);
                } else if self.heat_grid[neighbor.x][neighbor.y]
                    > self.heat_grid[pos_grid.x][pos_grid.y] + d
                {
                    self.heat_grid[neighbor.x][neighbor.y] =
                        self.heat_grid[pos_grid.x][pos_grid.y] + d;
                }
            }
        }

        // Calcul flow grid
        for (x, line) in self.flow_grid.iter_mut().enumerate() {
            for (y, value) in line.iter_mut().enumerate() {
                let mut flow = Vec2::ZERO;
                let mut min_heat = f32::MAX;
                for i in -1..=1 {
                    for j in -1..=1 {
                        if i != j {
                            let pos_x = x as i32 + i;
                            let pos_y = y as i32 + j;
                            if pos_x >= 0
                                && (pos_x as usize) < self.size_x
                                && pos_y >= 0
                                && (pos_y as usize) < self.size_y
                            {
                                let heat = self.heat_grid[pos_x as usize][pos_y as usize];
                                if heat > 0. {
                                    // flow += Vec2::new(i as f32, j as f32) / heat;
                                    if heat < min_heat {
                                        flow = Vec2::new(i as f32, j as f32);
                                        min_heat = heat;
                                    }
                                }
                            }
                        }
                    }
                }
                flow = flow.normalize_or_zero();
                *value = flow;
            }
        }
    }

    fn in_bound(&self, pos: &GridPos) -> bool {
        pos.x < self.size_x && pos.y < self.size_y
    }

    fn neighbors(&self, pos: &GridPos) -> HashSet<GridPos> {
        let possible_neighbors = vec![
            (pos.x.saturating_sub(1), pos.y.saturating_sub(1)),
            (pos.x, pos.y.saturating_sub(1)),
            (pos.x + 1, pos.y.saturating_sub(1)),
            (pos.x.saturating_sub(1), pos.y),
            (pos.x + 1, pos.y),
            (pos.x.saturating_sub(1), pos.y + 1),
            (pos.x, pos.y + 1),
            (pos.x + 1, pos.y + 1),
        ];

        possible_neighbors
            .into_iter()
            .map(|p| p.into())
            .filter(|pos| self.in_bound(pos))
            .collect()
    }

    pub fn trans_to_grid(&self, point: Vec2) -> Option<GridPos> {
        let grid_pos = (
            self.size_x * point.x as usize / self.window_width,
            self.size_y * point.y as usize / self.window_height,
        )
            .into();

        match self.in_bound(&grid_pos) {
            true => Some(grid_pos),
            false => None,
        }
    }
}

fn initialize_flow_field(
    // mut flowfield: ResMut<FlowField>,
    colliders: Query<(&Collider, &Transform), With<Wall>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
) {
    println!("Create fowfield");
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());
    let mut ff = FlowField::new(width as usize, height as usize);

    let mut positions = HashSet::new();
    positions.insert((ff.size_x / 2, ff.size_y / 2));

    ff.build(positions, colliders);

    // if cfg!(debug_assertions) {
    //     for (x, line) in ff.heat_grid.iter().enumerate() {
    //         for (y, value) in line.iter().enumerate() {
    //             commands.spawn_bundle(TextBundle {
    //                 style: Style {
    //                     align_self: AlignSelf::Center,
    //                     position_type: PositionType::Absolute,
    //                     position:
    //                     Rect {
    //                         bottom: Val::Px((y * BLOCK_SIZE_HEIGHT) as f32),
    //                         left: Val::Px((x * BLOCK_SIZE_WIDTH) as f32),
    //                         ..Default::default()
    //                     },
    //                     ..Default::default()
    //                 },
    //                 text: Text::with_section(
    //                     format!("{:.2}", value),
    //                     TextStyle {
    //                         font: asset_server.load("arial.ttf"),
    //                         font_size: 10.0,
    //                         color: Color::WHITE,
    //                     },
    //                     TextAlignment {
    //                         vertical: VerticalAlign::Center,
    //                         horizontal: HorizontalAlign::Center,
    //                     },
    //                 ),
    //                 ..Default::default()
    //             });
    //         }
    //     }
    // }

    commands.insert_resource(ff);

    // Test TODO("Delete this")

    // println!("Spawning square...");
    // let img = asset_server.load("square.png");
    // let mut rng = rand::thread_rng();
    // for _ in 0..100 {
    //     commands
    //         .spawn_bundle(SpriteBundle {
    //             sprite: Sprite {
    //                 color: Color::WHITE,
    //                 custom_size: Some(Vec2::new(3., 3.)),
    //                 ..Default::default()
    //             },
    //             transform: Transform {
    //                 translation: Vec3::new(
    //                     rng.gen_range(0.0..50.0),
    //                     rng.gen_range(0.0..50.0),
    //                     0.,
    //                 ),
    //                 ..Default::default()
    //             },
    //             texture: img.clone(),
    //             ..Default::default()
    //         })
    //         .insert(Collider::new(Box::new(Cuboid::new(Vector2::new(1.5, 1.5))), 0))
    //         .insert(Flowing)
    //         .insert(Rigidbody::zero(false))
    //         .insert(Trigger);
    // }
    // println!("Square spawned");
}

fn move_in_flow_field(
    flowfield: Res<FlowField>,
    mut query: Query<(&mut Rigidbody, &Transform), With<Flowing>>,
) {
    for (mut rb, trans) in query.iter_mut() {
        if let Some(grid_pos) = flowfield.trans_to_grid(trans.translation.truncate()) {
            let flow = flowfield.flow_grid[grid_pos.x][grid_pos.y];
            rb.velocity.x = flow.x;
            rb.velocity.y = flow.y;
        } else {
            println!("Error at pos : {:?}", trans.translation.truncate());
        }
    }
}

pub struct FlowFieldPlugin;

impl Plugin for FlowFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, initialize_flow_field)
            .add_system(move_in_flow_field.label("flowing"));
    }
}
