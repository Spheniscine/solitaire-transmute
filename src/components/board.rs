use dioxus::prelude::*;
use glam::Vec2;

use crate::{components::{CARD_BORDER_RADIUS_RATIO, CARD_FRAME_DEFAULT_COLOR, CARD_HEIGHT_RATIO, CardComponent, CardFrame, Movement, SkinTrait, rem}, game::{AnimationAct, AnimationKey, Board, BoardPos, Card, DepotRole, NUM_DEPOTS, Skin, Suit}};

#[component]
fn Circle(
    position: Vec2,
    size: f32,
    stroke: f32,
) -> Element {
    let stroke_width = stroke / size;
    rsx! {
        svg {
            style: "position: absolute; left: {rem(position.x)}; top: {rem(position.y)}; width: {rem(size)}; height: {rem(size)};",
            view_box: "0 0 1 1",
            xmlns: "http://www.w3.org/2000/svg",

            circle {
                cx: 0.5,
                cy: 0.5,
                r: 0.5 - stroke_width / 2.,
                stroke: CARD_FRAME_DEFAULT_COLOR,
                stroke_width,
                fill: "transparent",
            }
        }
    }
}

#[component]
fn Triangle(
    position: Vec2,
    size: f32,
    stroke: f32,
) -> Element {
    let stroke_width = stroke / size;
    let height_ratio = (3f64.sqrt() / 2.) as f32;

    let y1 = stroke_width / 2.;
    let y2 = height_ratio - y1;

    rsx! {
        svg {
            style: "position: absolute; left: {rem(position.x)}; top: {rem(position.y)}; width: {rem(size)};",
            view_box: "0 0 1 {height_ratio}",
            xmlns: "http://www.w3.org/2000/svg",

            polygon {
                points: "0.5,{y1} {1.-y1},{y2} {y1},{y2}",
                stroke: CARD_FRAME_DEFAULT_COLOR,
                stroke_width,
                fill: "transparent",
            }
        }
    }
}

#[component]
pub fn BoardComponent(
    position: Vec2,
    board: Board,
    skin: Skin,
    #[props(default)]
    onclick: EventHandler<BoardPos>,
    #[props(default)]
    ondoubleclick: EventHandler<BoardPos>,
    #[props(default)]
    oncontextmenu: EventHandler<BoardPos>,
    #[props(default)]
    animation_key: AnimationKey,
    #[props(default)]
    is_won: bool,
    #[props(default)]
    is_lost: bool,
) -> Element {
    let card_width = 11f32;
    let card_height = card_width * CARD_HEIGHT_RATIO;
    let spacer_x = 1f32;
    let start_x = 2f32;
    let start_y = 2f32;

    let pos_x = {
        let left = start_x;
        move |i: usize| {
            left + (card_width + spacer_x) * i as f32
        }
    };

    let pos_xr = {
        let right = 100. - start_x - card_width;
        move |i: usize| {
            right - (card_width + spacer_x) * i as f32
        }
    };

    let pos_xrm = pos_xr(0).midpoint(pos_xr(1));

    let foundation_y = start_y + 6f32;
    let engine_out_y = foundation_y + card_height + 6f32;
    let engine_in_y = engine_out_y + card_height + 1f32;
    let column_card_offset = Vec2::new(0., 6.);

    let get_pos = |depot: usize, ord: usize| {
        let (role, index) = DepotRole::role_and_subindex(depot).unwrap();
        match role {
            DepotRole::Tableau => 
                Vec2::new(pos_x(index), start_y) + column_card_offset * ord as f32,
            DepotRole::Foundation => Vec2::new(pos_xrm, foundation_y),
            DepotRole::EngineIn => Vec2::new(pos_xr(1 - index), engine_in_y),
            DepotRole::EngineOut => Vec2::new(pos_xrm, engine_out_y),
        }
    };

    let get_hint = |depot: usize| {
        let role = DepotRole::role(depot).unwrap();
        match role {
            DepotRole::Foundation => Some(skin.render_rank(&Card { rank: 1, suit: Suit::Spades })),
            _ => Some(rsx!{}),
        }
    };

    let selected_height = if let Some(BoardPos { depot_index, card_index }) = board.selected {
        let d = if DepotRole::role(depot_index).unwrap() == DepotRole::Tableau {
            board.depots[depot_index].len() - card_index - 1
        } else {
            0
        };

        card_height + column_card_offset.y * d as f32
    } else {0.};

    let stroke = 0.5f32 / 12. * 11.;

    let circle_decor = {
        let size = 18.;
        let position = get_pos(DepotRole::Foundation.id(0), 0)
            + Vec2::new(card_width, card_height) / 2. 
            - Vec2::new(size, size) / 2.;
        
        rsx! {
            Circle { 
                position,
                size, stroke,
            }
        }
    };

    let triangle_decor = {
        let size = card_width * 2. + spacer_x - 0.6;
        let position = Vec2::new(
            get_pos(DepotRole::EngineIn.id(0), 0).x + 0.15,
            get_pos(DepotRole::EngineOut.id(0), 0).y + 2.,
        );
        
        rsx! {
            Triangle { 
                position,
                size, stroke,
            }
        }
    };

    let moving_card = |p1: Vec2, p2: Vec2, card: Card| rsx! {
        Movement {
            src_translate_vec: p1 - p2,
            CardComponent {
                position: p2,
                width: card_width,
                card: card,
                skin,
            }
        }
    };

    let anims = board.animation_acts.iter().enumerate().map(|(i, act)| {
        match act {
            AnimationAct::Move { cards, pos1, pos2, rev } => {
                let mut pos1 = *pos1;
                let mut pos2 = *pos2;
                let rev = *rev;

                if rev { pos1.card_index += cards.len(); }
                let nodes = cards.iter().map(move |card| {
                    if rev { pos1.card_index -= 1; }
                    let p1 = get_pos(pos1.depot_index, pos1.card_index);
                    let p2 = get_pos(pos2.depot_index, pos2.card_index);
                    let res = moving_card(p1, p2, *card);
                    if !rev { pos1.card_index += 1; }
                    pos2.card_index += 1;
                    res
                });

                rsx! {
                    Fragment {
                        key: "{animation_key},{i}", // needed to force remounts, so animations don't get "stale" and refuse to replay
                        {nodes}
                    }
                }
            },
            AnimationAct::Combine { cards, .. } => {
                let p2 = get_pos(DepotRole::EngineOut.id(0), 0);
                let nodes = (0..cards.len()).map(|i| {
                    let p1 = get_pos(DepotRole::EngineIn.id(i), 0);
                    moving_card(p1, p2, cards[i])
                });
                rsx! {
                    Fragment {
                        key: "{animation_key},{i}", // needed to force remounts, so animations don't get "stale" and refuse to replay
                        {nodes}
                    }
                }
            },
        }
    });

    rsx! {
        div {
            position: "absolute",
            top: rem(position.y),
            left: rem(position.x),

            {circle_decor},
            {triangle_decor},

            for depot in 0..NUM_DEPOTS {
                if let Some(hint) = get_hint(depot) {
                    CardFrame { 
                        position: get_pos(depot, 0),
                        width: card_width,
                        hint,
                        onclick: move |_| {
                            onclick.call(BoardPos::new(depot, !0))
                        },
                        oncontextmenu: move |ev: Event<MouseData>| {
                            ev.prevent_default();
                            oncontextmenu.call(BoardPos::new(depot, !0))
                        },
                    }
                }

                for i in 0..board.depots[depot].len() {
                    if board.selected == Some(BoardPos::new(depot, i)) {
                        div {
                            position: "absolute",
                            top: rem(get_pos(depot, i).y),
                            left: rem(get_pos(depot, i).x),
                            width: rem(card_width),
                            height: rem(selected_height),
                            background_color: "#ff0",
                            border_radius: rem(card_width * CARD_BORDER_RADIUS_RATIO),
                            class: "selected-halo",
                        }
                    }

                    
                    CardComponent { 
                        position: get_pos(depot, i),
                        width: card_width,
                        card: board.depots[depot][i],
                        // number_hint: if !is_face_up(depot) {i + 1},
                        skin,
                        onclick: move |_| {
                            onclick.call(BoardPos::new(depot, i))
                        },
                        ondoubleclick: move |_| {
                            ondoubleclick.call(BoardPos::new(depot, i))
                        },
                        oncontextmenu: move |ev: Event<MouseData>| {
                            ev.prevent_default();
                            oncontextmenu.call(BoardPos::new(depot, i))
                        },
                    }
                }
            }

            {anims}

            if is_won {
                div {
                    position: "absolute",
                    top: rem(25.),
                    left: rem(17.5),
                    width: rem(59.),
                    background_color: "#505",
                    padding: rem(3.),
                    color: "#fff",
                    font_size: rem(7.),
                    border_radius: rem(2.),
                    text_align: "center",
                    "YOU WIN!",
                }
            }
        }
    }
}