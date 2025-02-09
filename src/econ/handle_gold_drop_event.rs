//Spawn gold pieces here
use bevy::prelude::*;

use super::components::GoldDropEvent;

// #[derive(Event)]
// pub struct GoldDropEvent {
//     pub drop_location: Transform,
//     pub amount: u32,
// }

pub fn on_gold_drop_event(trigger: Trigger<GoldDropEvent>) {
    //Only spawn a maximum of 10 new entities
    //10 gold = 10 "small" sprite
    //100 gold = 10 "medium" sprite
    //1101 gold = One Large, One Medium, One Small sprite
    //Etc.

    //Command.spawn(AutoLoot -> Loots when it touches the player
    //magenet -> scoots on the ground to the player
    //Sprite -> look of the gold itself, small, medium, large, xlarge, xxlarge piles sizes
    //Transform -> random location with 50 of the drop location, z-axis "grounded item"
    //GoldEffect -> current rotation of the coin and how much the sprite is glowing
    //Sensor -> Used to detect CollidingEntities with the player
    //CollidingEntities -> Used to figure out who is colliding with the coin
    //Magnet will use that colliding entites to scoot
    //Autoloot will collect the gold and despawn it when it's position is within 10 of the player
}
pub fn update_grounded_magnets() {
    //Any entity with magnet is attacked to their target entity (Just player for now)
}

pub fn update_grounded_autoloot_currency() {
    //Query for all items grounded with AutoLoot and Currency Tag
    //Place into wallet when positions overlap
}

pub fn update_grounded_autoloot_items() {
    //Query for all items grounded with AutoLoot and Currency Tag
    //Place into inventory if there is room
}
