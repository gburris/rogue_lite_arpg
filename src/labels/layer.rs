pub enum ZLayer {
    Ground,
    Player,
    WeaponBehindSprite,
    WeaponAboveSprite,
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
            ZLayer::WeaponBehindSprite => -0.4, //Thisis relative to the parent LMAO
            //So if the parent is 1.0, and this is attached, it was 1.6, always above player
            //-0.4 makes it "0.6 when on the screen"
            ZLayer::WeaponAboveSprite => 0.1, //Thisis relative to the parent LMAO
            //So if the parent is 1.0, and this is attached, it was 1.6, always above player
            //-0.4 makes it "0.6 when on the screen"
            ZLayer::Player => 1.0,
            ZLayer::Warpzone => 1.0,
            ZLayer::VisualEffect => 2.0,
        }
    }
}
