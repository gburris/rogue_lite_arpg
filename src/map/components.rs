use avian2d::prelude::*;
use bevy::prelude::*;

use crate::helpers::labels::GameCollisionLayer;

/**
 * Portals represent any "warping device" in the game, currently spawning a new zone when entered
 */
#[derive(Component)]
#[require(
    RigidBody(default_rigid_body),
    Collider(default_collider),
    CollisionLayers(default_collision_layers),
    Sensor
)]
pub enum Portal {
    StartingPortal,
    WarpZone,
}

fn default_collider() -> Collider {
    Collider::rectangle(100.0, 100.0)
}

fn default_rigid_body() -> RigidBody {
    RigidBody::Static
}

fn default_collision_layers() -> CollisionLayers {
    // Portals only collide with the player, and are sensors since we don't actually want collisions
    CollisionLayers::new(GameCollisionLayer::Portal, [GameCollisionLayer::Player])
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Water;

#[derive(Clone, Copy, PartialEq)]
pub enum TileType {
    Ground,
    Wall,
    Water,
}

#[derive(Clone)]
pub enum MapMarker {
    PlayerSpawn(Vec2), // Could even include spawn direction
    LevelExit(Vec2),
    EnemySpawns(Vec<Vec2>),
    BossSpawns(Vec<Vec2>),
    ChestSpawns(Vec<Vec2>),
}

#[derive(Clone)]
pub struct MapMarkers {
    pub markers: Vec<MapMarker>,
}

#[derive(Resource, Clone)]
pub struct MapLayout {
    pub tiles: Vec<Vec<TileType>>, // Your physical tile grid
    pub markers: Vec<MapMarker>,   // Collection of all logical positions
}

#[derive(Default)]
pub struct WallSection {
    pub start: (u32, u32),
    pub is_horizontal: bool,
    end: (u32, u32),
}

impl WallSection {
    pub fn new(start: (u32, u32), is_horizontal: bool) -> Self {
        WallSection {
            start,
            is_horizontal,
            end: start,
        }
    }

    pub fn extend(&mut self, pos: (u32, u32)) {
        self.end = pos;
    }

    pub fn length(&self) -> u32 {
        if self.is_horizontal {
            self.end.0 - self.start.0 + 1
        } else {
            self.end.1 - self.start.1 + 1
        }
    }
}
