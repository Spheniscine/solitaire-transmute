use dioxus::prelude::*;

use crate::{components::{CardText, VIDEO_GAMEPLAY, rem}, game::{Card, GameState, KATEX_SUITS_FONT_STR, ScreenState, Suit}};

#[component]
fn Emph(children: Element) -> Element {
    rsx! {
        strong {
            color: "#ff0",
            {children}
        }
    }
}

#[component]
pub fn Help(game_state: Signal<GameState>) -> Element {
    let st = game_state.read();
    let skin = st.skin;

    let stack_example = || {
        let mut ite = [
            Card { rank: 4, suit: Suit::Diamonds },
            Card { rank: 3, suit: Suit::Spades },
            Card { rank: 4, suit: Suit::Clubs },
            Card { rank: 3, suit: Suit::Hearts },
            Card { rank: 2, suit: Suit::Clubs },
        ].into_iter().map(|card| {
            rsx! {
                CardText { 
                    card, skin, color_mode: crate::game::ColorMode::Light,
                }
            }
        });


        let last = ite.next().unwrap();
        rsx! {
            {ite.next().unwrap()},
            for x in ite { "–", {x} },
            " can be placed on the ", {last}
        }
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; font-size: 3.75rem; color: #fff; padding: 4rem;",
            class: "help",

            div {
                text_align: "left",

                p {
                    margin_top: "0",
                    "The deck is a standard 52-card deck with 13 ranks and 4 suits.",
                }

                p {
                    "Cards in the ",Emph {"tableau"}," stack by ", Emph {"adjacent rank"}, " and " Emph {"unlike suit"}, ". Such stacks
                    of any size can be moved as a unit. (e.g. ",{stack_example()}")"
                }

                p {
                    "The ",Emph {"triangle"}," can be used to ",Emph {"combine"}," cards. Place 2 cards in the bottom slots, and 
                    you’ll get their combination in the top slot."
                    ul {
                        li { "A combined card can’t be combined again."}
                        li { "Combined cards have a ", Emph{"wild"}, " suit, which means its suit is ", Emph{"ignored"}, " by the stacking rule." }
                    }
                }

                p {
                    "The circled slot is the ",Emph {"foundation"},". To ",Emph {"win the game"},", you must pile cards from ",
                    span {
                        font_family: KATEX_SUITS_FONT_STR,
                        font_size: "1.2em",
                        "1"
                    }, " to ",
                    span {
                        font_family: KATEX_SUITS_FONT_STR,
                        font_size: "1.2em",
                        "26"
                    },
                    " in the foundation, regardless of suit."
                }

                p {
                    Emph{"Shortcut notes:"},

                    ul {
                        li {
                            "After selecting a stack, you may ", Emph {"right-click / long-press"}, " another tableau column to
                            stack in ", Emph {"reverse order"}, ". This shortcuts moving those cards one by one."
                        }

                        li {
                            Emph {"Double-clicking"}," a card will send it to the foundation."
                        }

                        li {
                            "A selected stack of cards may be sent to the foundation in one move, if the cards would fit in when moved
                            one by one."
                        }
                    }
                    
                    
                }

                div {
                    position: "absolute",
                    bottom: rem(2.),
                    width: "92rem",
                    display: "flex",
                    justify_content: "center",

                    a {
                        href: VIDEO_GAMEPLAY,
                        target: "_blank",
                        text_decoration: "none",
                        margin_right: rem(4.),
                        div {
                            width: rem(30.),
                            position: "relative",
                            class: "game-button",
                            "Example video"
                        }
                    }

                    div {
                        width: rem(30.),
                        position: "relative",
                        class: "game-button",
                        onclick: move |_| game_state.write().screen_state = ScreenState::Game,
                        "Back to game"
                    }
                }
            }
        }
    }
}