use dioxus::prelude::*;
use glam::Vec2;

use crate::{components::CardComponent, game::{Card, Skin, Suit}};

#[component]
pub fn Hero() -> Element {
    let mut skin = Skin::default();
    // skin.suits = crate::game::SuitSkin::Traditional;
    rsx! {
        CardComponent { 
            position: Vec2::new(10., 10.),
            width: 11.,
            card: Card { rank: 26, suit: Suit::Wild },
            skin,
        }
    }
}