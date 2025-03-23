pub enum ZLayer {
    Ground,
    OnGround,
    InAir,
    VisualEffect,

    BehindSprite,
    AboveSprite,
    LevelUpEffect,
}

impl ZLayer {
    pub fn z(&self) -> f32 {
        match self {
            ZLayer::Ground => -1.0,
            ZLayer::OnGround => 0.5,
            ZLayer::InAir => 1.0,
            ZLayer::VisualEffect => 2.0,

            // Z layer is additive in parent/child hierarchies
            // Parent 1 + child entity weapon of 0.1 = 1.1
            // These are the reletive z layers
            ZLayer::BehindSprite => -0.4,
            ZLayer::AboveSprite => 0.1,
            ZLayer::LevelUpEffect => -0.1,
        }
    }
}
