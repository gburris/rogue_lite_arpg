pub enum ZLayer {
    Ground,
    Player,
    Weapon,
    Enemy,
    Projectiles,
    Warpzone,
    VisualEffect,
}

impl ZLayer {
    pub fn z(&self) -> f32 {
        match self {
            ZLayer::Ground => 0.0,
            ZLayer::Projectiles => 0.4,
            ZLayer::Enemy => 0.5,
            ZLayer::Weapon => 0.6,
            ZLayer::Player => 1.0,
            ZLayer::Warpzone => 1.0,
            ZLayer::VisualEffect => 2.0,
        }
    }
}
