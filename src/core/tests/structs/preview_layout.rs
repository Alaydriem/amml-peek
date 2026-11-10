use crate::support::approx::Approx;
use peek_core::structs::{Point, PreviewLayout};

fn at(px: f32, py: f32, sw: f32, sh: f32) -> PreviewLayout {
    PreviewLayout::anchored(9, 3, Point { x: px, y: py }, Point { x: sw, y: sh })
}

#[test]
fn frame_size() {
    let layout = at(0.0, 300.0, 1000.0, 1000.0);
    assert!(Approx::near(layout.frame().width(), 9.0 * 18.0 + 8.0));
    assert!(Approx::near(layout.frame().height(), 3.0 * 18.0 + 8.0));
}

#[test]
fn anchors_above_right_when_there_is_room() {
    let layout = at(100.0, 200.0, 400.0, 300.0);
    assert!(Approx::rect(layout.frame(), 108.0, 130.0, 278.0, 192.0));
}

#[test]
fn falls_below_when_no_room_above() {
    let layout = at(100.0, 30.0, 400.0, 300.0);
    assert!(Approx::rect(layout.frame(), 108.0, 38.0, 278.0, 100.0));
}

#[test]
fn flips_to_left_of_pointer_near_right_edge() {
    let layout = at(390.0, 200.0, 400.0, 300.0);
    assert!(Approx::near(layout.frame().x1, 382.0));
    assert!(Approx::near(layout.frame().x0, 212.0));
}

#[test]
fn stays_right_when_it_just_fits() {
    let layout = at(222.0, 200.0, 400.0, 300.0);
    assert!(Approx::rect(layout.frame(), 230.0, 130.0, 400.0, 192.0));
}

#[test]
fn clamps_only_when_neither_side_fits() {
    let layout = at(50.0, 200.0, 100.0, 300.0);
    assert!(Approx::near(layout.frame().x0, 0.0));
    assert!(Approx::near(layout.frame().x1, 170.0));
}

#[test]
fn flips_left_when_pointer_is_at_the_very_right_edge() {
    let layout = at(640.0, 100.0, 640.0, 337.0);
    assert!(Approx::near(layout.frame().x1, 632.0));
    assert!(layout.frame().x0 >= 0.0);
}

#[test]
fn clamps_to_bottom_edge() {
    let layout = at(100.0, 50.0, 400.0, 100.0);
    assert!(Approx::near(layout.frame().y1, 100.0));
    assert!(Approx::near(layout.frame().y0, 38.0));
}

#[test]
fn cells_follow_row_major_slot_order() {
    let layout = at(0.0, 300.0, 1000.0, 1000.0);
    let x0 = layout.frame().x0 + PreviewLayout::PADDING;
    let y0 = layout.frame().y0 + PreviewLayout::PADDING;

    assert!(Approx::rect(layout.cell(0), x0, y0, x0 + 18.0, y0 + 18.0));
    assert!(Approx::rect(layout.cell(10), x0 + 18.0, y0 + 18.0, x0 + 36.0, y0 + 36.0));
    assert!(Approx::rect(layout.cell(26), x0 + 8.0 * 18.0, y0 + 2.0 * 18.0, x0 + 9.0 * 18.0, y0 + 3.0 * 18.0));

    assert!(Approx::near(layout.cell(26).x1 + PreviewLayout::PADDING, layout.frame().x1));
    assert!(Approx::near(layout.cell(26).y1 + PreviewLayout::PADDING, layout.frame().y1));
}

#[test]
fn icon_is_centred_in_cell() {
    let layout = at(0.0, 300.0, 1000.0, 1000.0);
    let cell = layout.cell(5);
    let icon = layout.icon(5);
    assert!(Approx::near(icon.width(), 16.0));
    assert!(Approx::near(icon.height(), 16.0));
    assert!(Approx::near(icon.x0 - cell.x0, 1.0));
    assert!(Approx::near(icon.y0 - cell.y0, 1.0));
    assert!(Approx::near(cell.x1 - icon.x1, 1.0));
    assert!(Approx::near(cell.y1 - icon.y1, 1.0));
}

#[test]
fn frame_stays_inside_screen_and_off_the_pointer() {
    let (w, h) = (320.0, 240.0);
    let pointers = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h), (w / 2.0, 0.0), (w / 2.0, h), (0.0, h / 2.0), (w, h / 2.0)];
    for (x, y) in pointers {
        let pointer = Point { x, y };
        let frame = at(x, y, w, h).frame();
        assert!(frame.x0 >= 0.0 && frame.y0 >= 0.0 && frame.x1 <= w && frame.y1 <= h, "outside screen at ({x}, {y})");
        assert!(!frame.contains(pointer), "contains pointer at ({x}, {y})");
    }
}

#[test]
fn frame_clamps_to_origin_on_a_tiny_screen() {
    let frame = at(50.0, 25.0, 100.0, 50.0).frame();
    assert!(Approx::near(frame.x0, 0.0));
    assert!(Approx::near(frame.y0, 0.0));
}

fn panel(strip_rows: usize, hint: bool, px: f32, py: f32, sw: f32, sh: f32) -> PreviewLayout {
    PreviewLayout::anchored_panel(9, 3, strip_rows, hint, Point { x: px, y: py }, Point { x: sw, y: sh })
}

#[test]
fn a_strip_and_a_hint_make_the_panel_taller_by_the_gap_and_the_rows() {
    let plain = at(0.0, 500.0, 1000.0, 1000.0);
    let strip = panel(2, false, 0.0, 500.0, 1000.0, 1000.0);
    let both = panel(2, true, 0.0, 500.0, 1000.0, 1000.0);

    assert!(Approx::near(strip.frame().height() - plain.frame().height(), 8.0 + 2.0 * 18.0));
    assert!(Approx::near(both.frame().height() - strip.frame().height(), 4.0 + 8.0));
    assert!(Approx::near(both.frame().width(), plain.frame().width()));
}

#[test]
fn the_strip_sits_under_the_grid_after_the_gap_and_the_hint_under_the_strip() {
    let layout = panel(1, true, 0.0, 500.0, 1000.0, 1000.0);
    let frame = layout.frame();

    let first = layout.strip_cell(0).unwrap();
    assert!(Approx::near(first.y0, layout.cell(26).y1 + 8.0));
    assert!(Approx::near(first.x0, layout.cell(0).x0));
    let ninth = layout.strip_cell(8).unwrap();
    assert!(Approx::near(ninth.x1, layout.cell(26).x1));
    assert!(layout.strip_cell(9).unwrap().y0 > first.y0);
    assert!(Approx::near(layout.strip_icon(0).unwrap().x0 - first.x0, 1.0));
    let hint = layout.hint().unwrap();
    assert!(Approx::near(hint.y0, first.y1 + 4.0));
    assert!(Approx::near(hint.y1 + PreviewLayout::PADDING, frame.y1));
    assert!(Approx::near(hint.x0, first.x0));
}

#[test]
fn without_a_strip_or_a_hint_there_are_none() {
    let layout = at(0.0, 300.0, 1000.0, 1000.0);

    assert_eq!(layout.strip_cell(0), None);
    assert_eq!(layout.strip_icon(0), None);
    assert_eq!(layout.hint(), None);
}

#[test]
fn a_strip_has_no_cell_from_index_64() {
    let layout = panel(8, false, 0.0, 500.0, 1000.0, 1000.0);

    assert!(layout.strip_cell(63).is_some());
    assert_eq!(layout.strip_cell(64), None);
}

#[test]
fn the_taller_panel_goes_below_the_pointer_when_it_does_not_fit_above() {
    let tall = panel(3, true, 100.0, 120.0, 400.0, 400.0);
    let short = at(100.0, 120.0, 400.0, 400.0);

    assert!(Approx::near(short.frame().y1, 112.0));
    assert!(Approx::near(tall.frame().y0, 128.0));
}

#[test]
fn the_taller_panel_flips_left_near_the_right_edge() {
    let tall = panel(3, true, 390.0, 300.0, 400.0, 400.0);

    assert!(Approx::near(tall.frame().x1, 382.0));
    assert!(Approx::near(tall.frame().y1, 292.0));
}

#[test]
fn the_taller_panel_near_the_top_right_corner_stays_inside_and_off_the_pointer() {
    let pointer = Point { x: 395.0, y: 5.0 };
    let frame = panel(8, true, 395.0, 5.0, 400.0, 300.0).frame();

    assert!(frame.x0 >= 0.0 && frame.y0 >= 0.0 && frame.x1 <= 400.0 && frame.y1 <= 300.0);
    assert!(!frame.contains(pointer));
}

#[test]
fn a_panel_taller_than_the_screen_clamps_to_the_top() {
    let tall = panel(8, true, 100.0, 100.0, 400.0, 150.0);

    assert!(Approx::near(tall.frame().y0, 0.0));
}

#[test]
fn the_strip_draws_whole_rows_of_cells_up_to_64() {
    assert_eq!(panel(0, false, 0.0, 500.0, 1000.0, 1000.0).strip_cells(), 0);
    assert_eq!(panel(1, false, 0.0, 500.0, 1000.0, 1000.0).strip_cells(), 9);
    assert_eq!(panel(8, false, 0.0, 500.0, 1000.0, 1000.0).strip_cells(), 64);
}
