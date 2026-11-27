use std::cell::Cell;
use std::rc::Rc;

use alaydriem_peek::services::PreviewPanel;
use alaydriem_peek::types::{BoxSummary, DrawOutcome, ManagerDelta, ViewFrame};
use peek_core::structs::{HoveredSlot, Point, PreviewLayout, Rect, Rgb, SlotCollection};

use crate::support::{CanvasOp, FakeBox, FakeEntry, FakeResolver, RecordingCanvas};

const SCREEN: u64 = 0x10;
const OTHER: u64 = 0x20;

fn frame(screen: u64) -> ViewFrame {
    ViewFrame { screen, pointer: Point { x: 200.0, y: 150.0 }, size: Point { x: 400.0, y: 300.0 } }
}

fn panel() -> PreviewPanel<FakeBox> {
    PreviewPanel::new()
}

#[test]
fn a_screen_that_is_not_the_hovered_ones_draws_nothing_and_resolves_nothing() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    let mut canvas = RecordingCanvas::default();
    panel.hovered(SCREEN, SlotCollection::Container, 3);

    let outcome = panel.draw(&frame(OTHER), &resolver, &mut canvas);

    assert_eq!(outcome, DrawOutcome::Idle);
    assert!(resolver.loaded().is_empty());
    assert!(canvas.ops.is_empty());
}

#[test]
fn the_hovered_box_draws_the_panel_next_to_the_pointer() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    let mut canvas = RecordingCanvas::default();
    panel.hovered(SCREEN, SlotCollection::Container, 3);

    let outcome = panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(outcome, DrawOutcome::Drawn);
    let expected = PreviewLayout::anchored(9, 3, Point { x: 200.0, y: 150.0 }, Point { x: 400.0, y: 300.0 });
    assert!(matches!(&canvas.ops[0], CanvasOp::Fill { area, .. } if *area == expected.frame()));
    let asked = resolver.loaded();
    assert_eq!(asked, [HoveredSlot { screen: SCREEN, collection: SlotCollection::Container, index: 3 }]);
}

#[test]
fn an_empty_box_draws_the_frame_and_27_cells_and_no_items() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![]));
    let mut panel = panel();
    let mut canvas = RecordingCanvas::default();
    panel.hovered(SCREEN, SlotCollection::Container, 0);

    let outcome = panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(outcome, DrawOutcome::Drawn);
    assert_eq!(canvas.ops.len(), 29);
    assert!(canvas.ops.iter().all(|op| matches!(op, CanvasOp::Fill { .. } | CanvasOp::Outline { .. })));
}

#[test]
fn the_screen_closing_ends_the_hover() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    let mut canvas = RecordingCanvas::default();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    panel.left(SCREEN);

    let outcome = panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(outcome, DrawOutcome::Idle);
    assert!(resolver.loaded().is_empty());
    assert!(canvas.ops.is_empty());
}

#[test]
fn a_hover_of_box_b_before_the_unhover_of_box_a_keeps_b() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    let mut canvas = RecordingCanvas::default();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    panel.hovered(SCREEN, SlotCollection::Container, 4);
    panel.unhovered(SCREEN, SlotCollection::Container, 3);

    let outcome = panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(outcome, DrawOutcome::Drawn);
    assert_eq!(resolver.loaded()[0].index, 4);
}

#[test]
fn a_slot_that_holds_no_shulker_box_draws_nothing_and_drops_the_old_stacks() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let mut first = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &resolver, &mut first);
    assert_eq!(drops.get(), 0);
    resolver.set(None);
    let mut second = RecordingCanvas::default();

    let outcome = panel.draw(&frame(SCREEN), &resolver, &mut second);

    assert_eq!(outcome, DrawOutcome::Skipped);
    assert!(second.ops.is_empty());
    assert_eq!(drops.get(), 1);
}

#[test]
fn a_disabled_event_drops_the_stacks_at_once_and_an_enabled_event_does_not() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let mut canvas = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    panel.apply(ManagerDelta { on: Some(true), disabled: false });
    assert_eq!(drops.get(), 0);
    assert!(panel.is_tracking(SCREEN));

    panel.apply(ManagerDelta { on: Some(false), disabled: true });

    assert_eq!(drops.get(), 1);
    assert!(!panel.is_tracking(SCREEN));
}

#[test]
fn clear_forgets_the_hover_and_drops_the_stacks() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let mut first = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &resolver, &mut first);

    panel.clear();

    let mut second = RecordingCanvas::default();
    assert_eq!(panel.draw(&frame(SCREEN), &resolver, &mut second), DrawOutcome::Idle);
    assert_eq!(drops.get(), 1);
    assert!(!panel.is_tracking(SCREEN));
}

// two bundles, at slots 0 and 4, and a stone between
fn bundles() -> Vec<FakeEntry> {
    vec![
        FakeEntry::item(Some(0), "bundle")
            .bundle(Some(16), Some(vec![FakeEntry::item(Some(0), "a"), FakeEntry::item(Some(1), "b")])),
        FakeEntry::item(Some(2), "stone"),
        FakeEntry::item(Some(4), "bundle").bundle(Some(48), Some(vec![FakeEntry::item(Some(0), "c")])),
    ]
}

fn scrolling_panel() -> PreviewPanel<FakeBox> {
    let mut panel = panel();
    panel.set_scroll(true);
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    panel
}

fn drawn(panel: &mut PreviewPanel<FakeBox>, resolver: &FakeResolver) -> RecordingCanvas {
    let mut canvas = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), resolver, &mut canvas);
    canvas
}

fn labels(canvas: &RecordingCanvas) -> Vec<&str> {
    canvas
        .ops
        .iter()
        .filter_map(|op| match op {
            CanvasOp::Label { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

// the items drawn under the grid: the contents of the selected bundle
fn strip_items(canvas: &RecordingCanvas, layout: &PreviewLayout) -> Vec<String> {
    let grid_bottom = layout.cell(26).y1;
    canvas
        .ops
        .iter()
        .filter_map(|op| match op {
            CanvasOp::Item { name, icon, .. } if icon.y0 > grid_bottom => Some(name.clone()),
            _ => None,
        })
        .collect()
}

fn outlined_cells(canvas: &RecordingCanvas) -> Vec<Rect> {
    canvas
        .ops
        .iter()
        .filter_map(|op| match op {
            CanvasOp::Outline { area, color, .. } if *color == Rgb { r: 1.0, g: 1.0, b: 1.0 } => Some(*area),
            _ => None,
        })
        .collect()
}

fn panel_layout(strip_rows: usize, hint: bool) -> PreviewLayout {
    PreviewLayout::anchored_panel(9, 3, strip_rows, hint, Point { x: 200.0, y: 150.0 }, Point { x: 400.0, y: 300.0 })
}

#[test]
fn the_first_bundle_is_selected_and_outlined_and_its_strip_and_the_hint_are_drawn() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();

    let canvas = drawn(&mut panel, &resolver);

    let layout = panel_layout(1, true);
    assert!(matches!(&canvas.ops[0], CanvasOp::Fill { area, .. } if *area == layout.frame()));
    assert_eq!(outlined_cells(&canvas), [layout.cell(0)]);
    assert_eq!(strip_items(&canvas, &layout), ["a", "b"]);
    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (1/2)"]);
}

#[test]
fn without_the_wheel_hook_the_first_bundle_shows_and_no_hint_is_drawn() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);

    let canvas = drawn(&mut panel, &resolver);

    assert_eq!(strip_items(&canvas, &panel_layout(1, false)), ["a", "b"]);
    assert!(labels(&canvas).is_empty());
    assert!(!panel.scroll(Some(SCREEN), true, 1));
}

#[test]
fn a_box_with_one_bundle_shows_its_strip_and_no_hint() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()[..2].to_vec()));
    let mut panel = scrolling_panel();

    let canvas = drawn(&mut panel, &resolver);

    assert_eq!(strip_items(&canvas, &panel_layout(1, false)), ["a", "b"]);
    assert!(labels(&canvas).is_empty());
    assert!(!panel.scroll(Some(SCREEN), true, 1));
}

#[test]
fn a_box_without_bundles_draws_the_plain_panel() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = scrolling_panel();

    let canvas = drawn(&mut panel, &resolver);

    let expected = PreviewLayout::anchored(9, 3, Point { x: 200.0, y: 150.0 }, Point { x: 400.0, y: 300.0 });
    assert!(matches!(&canvas.ops[0], CanvasOp::Fill { area, .. } if *area == expected.frame()));
    assert!(outlined_cells(&canvas).is_empty());
    assert!(labels(&canvas).is_empty());
}

#[test]
fn a_notch_with_shift_selects_the_next_bundle_drops_the_old_strip_at_once_and_wraps_both_ways() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);
    assert_eq!(drops.get(), 0);

    assert!(panel.scroll(Some(SCREEN), true, 1));

    assert_eq!(drops.get(), 2);
    let canvas = drawn(&mut panel, &resolver);
    assert_eq!(strip_items(&canvas, &panel_layout(1, true)), ["c"]);
    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (2/2)"]);
    assert_eq!(outlined_cells(&canvas), [panel_layout(1, true).cell(4)]);

    assert!(panel.scroll(Some(SCREEN), true, 1));
    let canvas = drawn(&mut panel, &resolver);
    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (1/2)"]);

    assert!(panel.scroll(Some(SCREEN), true, -1));
    let canvas = drawn(&mut panel, &resolver);
    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (2/2)"]);
}

#[test]
fn a_notch_that_is_not_the_goal_case_is_not_taken_and_the_selection_stays() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);

    assert!(!panel.scroll(Some(SCREEN), false, 1));
    assert!(!panel.scroll(Some(OTHER), true, 1));
    assert!(!panel.scroll(None, true, 1));
    assert!(!panel.scroll(Some(SCREEN), true, 0));

    assert_eq!(drops.get(), 0);
    let canvas = drawn(&mut panel, &resolver);
    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (1/2)"]);
}

#[test]
fn another_box_starts_again_at_the_first_bundle_and_drops_the_old_stacks() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);
    panel.scroll(Some(SCREEN), true, 1);
    drawn(&mut panel, &resolver);
    let before = drops.get();

    resolver.next_box();
    let canvas = drawn(&mut panel, &resolver);

    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (1/2)"]);
    assert_eq!(strip_items(&canvas, &panel_layout(1, true)), ["a", "b"]);
    // the three box stacks and the one strip stack of the old box
    assert_eq!(drops.get(), before + 3 + 1);
}

#[test]
fn leaving_the_slot_and_hovering_the_same_box_again_starts_at_the_first_bundle() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);
    panel.scroll(Some(SCREEN), true, 1);
    drawn(&mut panel, &resolver);

    panel.unhovered(SCREEN, SlotCollection::Container, 3);
    let idle = drawn(&mut panel, &resolver);
    assert!(idle.ops.is_empty());
    assert!(!panel.scroll(Some(SCREEN), true, 1));
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let canvas = drawn(&mut panel, &resolver);

    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (1/2)"]);
    assert_eq!(strip_items(&canvas, &panel_layout(1, true)), ["a", "b"]);
}

#[test]
fn a_notch_after_moving_to_another_slot_before_the_next_draw_is_not_taken() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);

    panel.hovered(SCREEN, SlotCollection::Container, 7);

    assert!(!panel.scroll(Some(SCREEN), true, 1));
}

#[test]
fn hovering_the_same_slot_again_keeps_the_selection() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);
    panel.scroll(Some(SCREEN), true, 1);

    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let canvas = drawn(&mut panel, &resolver);

    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (2/2)"]);
}

#[test]
fn another_screen_closing_keeps_the_selection_and_the_stacks() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);
    panel.scroll(Some(SCREEN), true, 1);
    drawn(&mut panel, &resolver);
    let before = drops.get();

    panel.left(OTHER);
    let canvas = drawn(&mut panel, &resolver);

    assert_eq!(drops.get(), before);
    assert_eq!(labels(&canvas), ["Shift + scroll: next bundle (2/2)"]);
}

#[test]
fn a_notch_after_the_screen_closed_is_not_taken() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);

    panel.left(SCREEN);

    assert!(!panel.scroll(Some(SCREEN), true, 1));
}

#[test]
fn a_hovered_slot_that_holds_no_box_ends_the_selection() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let nothing = FakeResolver::new(&drops, None);
    nothing.next_box();
    let mut panel = scrolling_panel();
    drawn(&mut panel, &resolver);

    let mut canvas = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &nothing, &mut canvas);

    assert!(!panel.scroll(Some(SCREEN), true, 1));
}

#[test]
fn the_key_is_read_every_frame_and_the_box_only_when_the_key_changes() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);

    for _ in 0..3 {
        let mut canvas = RecordingCanvas::default();
        panel.draw(&frame(SCREEN), &resolver, &mut canvas);
    }

    assert_eq!(resolver.keys(), 3);
    assert_eq!(resolver.loaded().len(), 1);
    resolver.next_box();
    let mut canvas = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);
    assert_eq!(resolver.loaded().len(), 2);
    assert_eq!(drops.get(), 1);
}

#[test]
fn a_loaded_box_is_reported_once_with_its_items_and_bundles() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(bundles()));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let mut canvas = RecordingCanvas::default();

    panel.draw(&frame(SCREEN), &resolver, &mut canvas);
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(panel.take_loaded(), Some(BoxSummary { items: 3, bundles: 2 }));
    assert_eq!(panel.take_loaded(), None);
}

#[test]
fn a_box_is_read_again_after_the_pointer_left_every_slot() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let mut canvas = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);
    panel.unhovered(SCREEN, SlotCollection::Container, 3);
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);
    assert_eq!(drops.get(), 1);

    panel.hovered(SCREEN, SlotCollection::Container, 3);
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(resolver.loaded().len(), 2);
}

#[test]
fn a_view_of_another_screen_does_not_drop_the_box_of_the_hovered_one() {
    let drops = Rc::new(Cell::new(0));
    let resolver = FakeResolver::new(&drops, Some(vec![FakeEntry::item(Some(0), "stone")]));
    let mut panel = panel();
    panel.hovered(SCREEN, SlotCollection::Container, 3);
    let mut canvas = RecordingCanvas::default();
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    panel.draw(&frame(OTHER), &resolver, &mut canvas);
    panel.draw(&frame(SCREEN), &resolver, &mut canvas);

    assert_eq!(resolver.loaded().len(), 1);
    assert_eq!(drops.get(), 0);
}
