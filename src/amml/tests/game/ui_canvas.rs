use std::sync::Arc;

use alaydriem_peek::game::{GameBox, UiCanvas};
use alaydriem_peek::traits::{BoxSource, Canvas};
use amml_api::ItemDrawHandle;
use amml_api::ScreenRenderExt;
use peek_core::structs::{Rect, Rgb};

use crate::support::{PeekRig, ScreenProbe};

fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect { x0, y0, x1, y1 }
}

fn chest() -> amml_api::testing::FakeScreen {
    PeekRig::chest(vec![Some(PeekRig::shulker(vec![PeekRig::item(0, "minecraft:stone", 2)]))])
}

// the stack handle of the one entry of the box, kept after the render call
fn stack_of(game: &amml_api::testing::FakeGame) -> Arc<ItemDrawHandle> {
    ScreenProbe::run(game, &chest(), |event| {
        let container = event.container_screen().unwrap();
        let stack = container.visual_item(PeekRig::container(0)).unwrap();
        GameBox::new(stack.container_contents().unwrap()).build(0).unwrap().stack
    })
}

#[test]
fn fills_outlines_and_items_draw_at_the_places_of_the_rectangles() {
    let game = PeekRig::offered();
    game.open_gate();
    let stack = stack_of(&game);

    ScreenProbe::run(&game, &chest(), move |event| {
        let draw = event.draw().unwrap();
        let mut canvas = UiCanvas::new(&draw);
        canvas.fill(rect(10.0, 20.0, 40.0, 60.0), Rgb { r: 0.5, g: 0.5, b: 0.5 }, 1.0);
        canvas.outline(rect(1.0, 2.0, 11.0, 22.0), Rgb { r: 1.0, g: 1.0, b: 1.0 }, 1.0, 1);
        canvas.item(&stack, rect(5.0, 6.0, 21.0, 22.0), false, 1.0, 17);
        canvas.item(&stack, rect(5.0, 6.0, 21.0, 22.0), true, 1.35, 17);
    });

    let draws = game.draws();
    assert_eq!(draws[0], "fill 10 20 30 40");
    assert_eq!(draws[1], "outline 1 2 10 20");
    assert!(draws[2].starts_with("item minecraft:stone 5 6 "), "{draws:?}");
    assert!(draws[3].starts_with("item minecraft:stone 5 6 "), "{draws:?}");
    assert_ne!(draws[2], draws[3]);
}

#[test]
fn a_count_sits_at_the_bottom_of_the_cell_and_a_label_at_the_top_of_the_area() {
    let game = PeekRig::offered();
    game.open_gate();

    let texts = ScreenProbe::run(&game, &chest(), |event| {
        let draw = event.draw().unwrap();
        let mut canvas = UiCanvas::new(&draw);
        canvas.count("64", rect(10.0, 20.0, 26.0, 36.0));
        canvas.label("hint", rect(10.0, 50.0, 90.0, 60.0));
        canvas.texts()
    });

    assert_eq!(texts, 2);
    let draws = game.draws();
    assert_eq!(draws[0], "text \"64\" 10 29");
    assert_eq!(draws[1], "text \"hint\" 10 50");
}

#[test]
fn an_item_of_a_world_that_was_left_marks_the_canvas_stale() {
    let game = PeekRig::offered();
    game.open_gate();
    let stack = stack_of(&game);
    game.leave_item_world();

    let stale = ScreenProbe::run(&game, &chest(), move |event| {
        let draw = event.draw().unwrap();
        let mut canvas = UiCanvas::new(&draw);
        let before = canvas.stale();
        canvas.item(&stack, rect(0.0, 0.0, 16.0, 16.0), false, 1.0, 17);
        (before, canvas.stale())
    });

    assert_eq!(stale, (false, true));
    assert!(game.draws().is_empty());
}
