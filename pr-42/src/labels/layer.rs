pub enum ZLayer {
    Ground,
    Player,
    WeaponBehindSprite,
    WeaponAboveSprite,
    Enemy,
    Projectiles,
    Exit,
    VisualEffect,
    LevelUpEffect,
    ItemOnGround,
}

impl ZLayer {
    pub fn z(&self) -> f32 {
        match self {
            ZLayer::Ground => 0.0,
            ZLayer::Projectiles => 0.4,
            ZLayer::Enemy => 0.5,
            //Z layer is additive in parent:child hieracrchies
            //Parent 1 + child entity weapon of 0.1 = 1.1
            ZLayer::WeaponBehindSprite => -0.4,
            ZLayer::WeaponAboveSprite => 0.1,
            ZLayer::ItemOnGround => 0.4,
            ZLayer::LevelUpEffect => -0.1,
            ZLayer::Player => 1.0,
            ZLayer::Exit => 1.0,
            ZLayer::VisualEffect => 2.0,
        }
    }
}
