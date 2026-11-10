use std::cell::Cell;
use std::rc::Rc;

use alaydriem_peek::services::{PreviewCache, PreviewRenderer};
use alaydriem_peek::types::PreviewKey;
use peek_core::services::FillBar;
use peek_core::structs::{DurabilityBar, Point, PreviewLayout, Rect, Rgb};

use crate::support::{CanvasOp, FakeBox, FakeEntry, RecordingCanvas, TrackedStack};

fn layout() -> PreviewLayout {
    PreviewLayout::anchored(9, 3, Point { x: 200.0, y: 150.0 }, Point { x: 400.0, y: 300.0 })
}

fn cache_of(entries: Vec<FakeEntry>) -> (PreviewCache<TrackedStack>, Rc<Cell<usize>>) {
    let drops = Rc::new(Cell::new(0));
    let source = FakeBox::new(&drops, entries);
    let mut cache = PreviewCache::new();
    cache.sync(PreviewKey { stack: 1, user_data: 2, count: 1 }, Some(&source));
    (cache, drops)
}

fn rgb(r: f32, g: f32, b: f32) -> Rgb {
    Rgb { r, g, b }
}

#[test]
fn an_empty_box_draws_the_frame_27_cells_and_the_border_and_nothing_else() {
    let layout = layout();
    let (cache, _drops) = cache_of(vec![]);
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, None, None, &mut canvas);

    let mut expected = vec![CanvasOp::Fill { area: layout.frame(), color: rgb(0.10, 0.10, 0.10), alpha: 0.92 }];
    for slot in 0..27 {
        expected.push(CanvasOp::Fill { area: layout.icon(slot), color: rgb(0.23, 0.23, 0.23), alpha: 1.0 });
    }
    expected.push(CanvasOp::Outline { area: layout.frame(), color: rgb(0.55, 0.35, 0.70), alpha: 1.0, thickness: 1 });
    assert_eq!(canvas.ops, expected);
}

#[test]
fn items_bars_and_counts_follow_the_fills_in_that_order_and_the_text_is_flushed_last() {
    let layout = layout();
    let (cache, _drops) = cache_of(vec![
        FakeEntry::item(Some(0), "stone").counted("64"),
        FakeEntry::item(Some(1), "book").glinting(),
        FakeEntry::item(Some(2), "pickaxe").damaged(100, 200),
    ]);
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, None, None, &mut canvas);

    let tail = &canvas.ops[29..];
    let bar = DurabilityBar::new(true, 100, 200).unwrap();
    let item = |name: &str, slot: usize, foil: bool, alpha: f32| CanvasOp::Item {
        name: name.to_string(),
        icon: layout.icon(slot),
        foil,
        alpha,
        z_order: 17,
    };
    let expected = vec![
        item("stone", 0, false, 1.0),
        item("book", 1, false, 1.0),
        item("book", 1, true, 1.35),
        item("pickaxe", 2, false, 1.0),
        CanvasOp::Fill { area: bar.background(layout.icon(2)), color: rgb(0.0, 0.0, 0.0), alpha: 1.0 },
        CanvasOp::Fill { area: bar.foreground(layout.icon(2)), color: bar.color(), alpha: 1.0 },
        CanvasOp::Count { text: "64".to_string(), cell: layout.cell(0) },
        CanvasOp::FlushText,
    ];
    assert_eq!(tail, expected.as_slice());
}

#[test]
fn the_glint_pass_is_drawn_only_for_a_glint_stack() {
    let layout = layout();
    let (cache, _drops) = cache_of(vec![FakeEntry::item(Some(0), "stone"), FakeEntry::item(Some(1), "apple")]);
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, None, None, &mut canvas);

    let foil: Vec<bool> = canvas
        .ops
        .iter()
        .filter_map(|op| match op {
            CanvasOp::Item { foil, .. } => Some(*foil),
            _ => None,
        })
        .collect();
    assert_eq!(foil, [false, false]);
}

#[test]
fn a_nearly_broken_item_keeps_the_black_strip_and_has_no_fill() {
    let layout = layout();
    let (cache, _drops) = cache_of(vec![FakeEntry::item(Some(0), "pickaxe").damaged(199, 200)]);
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, None, None, &mut canvas);

    let bar = DurabilityBar::new(true, 199, 200).unwrap();
    assert_eq!(bar.foreground(layout.icon(0)).width(), 0.0);
    let fills: Vec<&CanvasOp> = canvas.ops[29..].iter().filter(|op| matches!(op, CanvasOp::Fill { .. })).collect();
    assert_eq!(fills.len(), 1);
    assert_eq!(
        *fills[0],
        CanvasOp::Fill { area: bar.background(layout.icon(0)), color: rgb(0.0, 0.0, 0.0), alpha: 1.0 }
    );
}

#[test]
fn text_is_flushed_once_and_only_when_a_count_was_queued() {
    let layout = layout();
    let (cache, _drops) =
        cache_of(vec![FakeEntry::item(Some(0), "stone").counted("12"), FakeEntry::item(Some(5), "dirt").counted("3")]);
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, None, None, &mut canvas);

    let flushes = canvas.ops.iter().filter(|op| **op == CanvasOp::FlushText).count();
    assert_eq!(flushes, 1);
    assert_eq!(canvas.ops.last(), Some(&CanvasOp::FlushText));
    let counts: Vec<&CanvasOp> = canvas.ops.iter().filter(|op| matches!(op, CanvasOp::Count { .. })).collect();
    assert_eq!(counts.len(), 2);

    let (plain, _drops) = cache_of(vec![FakeEntry::item(Some(0), "stone")]);
    let mut quiet = RecordingCanvas::default();
    PreviewRenderer::draw(&layout, &plain, None, None, &mut quiet);
    assert!(!quiet.ops.contains(&CanvasOp::FlushText));
}

fn panel_layout(strip_rows: usize, hint: bool) -> PreviewLayout {
    PreviewLayout::anchored_panel(9, 3, strip_rows, hint, Point { x: 200.0, y: 150.0 }, Point { x: 400.0, y: 300.0 })
}

fn bundle_cache() -> (PreviewCache<TrackedStack>, FakeBox, Rc<Cell<usize>>) {
    let drops = Rc::new(Cell::new(0));
    let inside =
        vec![FakeEntry::item(Some(0), "stone").counted("5"), FakeEntry::item(Some(1), "pickaxe").damaged(100, 200)];
    let source = FakeBox::new(
        &drops,
        vec![
            FakeEntry::item(Some(0), "pickaxe").damaged(100, 200),
            FakeEntry::item(Some(1), "bundle").bundle(Some(32), Some(inside)),
        ],
    );
    let mut cache = PreviewCache::new();
    cache.sync(PreviewKey { stack: 1, user_data: 2, count: 1 }, Some(&source));
    cache.sync_strip(Some(&source), Some(1));
    (cache, source, drops)
}

#[test]
fn the_panel_with_a_strip_draws_in_the_documented_order() {
    let layout = panel_layout(1, true);
    let (cache, _source, _drops) = bundle_cache();
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, Some(1), Some("Shift + scroll: next bundle (1/2)"), &mut canvas);

    let bar = DurabilityBar::new(true, 100, 200).unwrap();
    let item =
        |name: &str, icon: Rect| CanvasOp::Item { name: name.to_string(), icon, foil: false, alpha: 1.0, z_order: 17 };
    let black = rgb(0.0, 0.0, 0.0);
    let fill = |area: Rect, color: Rgb| CanvasOp::Fill { area, color, alpha: 1.0 };
    let icon = layout.icon(1);
    let strip_icon = |position: usize| layout.strip_icon(position).unwrap();
    let mut expected = vec![CanvasOp::Fill { area: layout.frame(), color: rgb(0.10, 0.10, 0.10), alpha: 0.92 }];
    for slot in 0..27 {
        expected.push(fill(layout.icon(slot), rgb(0.23, 0.23, 0.23)));
    }
    for position in 0..9 {
        expected.push(fill(strip_icon(position), rgb(0.23, 0.23, 0.23)));
    }
    expected.push(CanvasOp::Outline { area: layout.frame(), color: rgb(0.55, 0.35, 0.70), alpha: 1.0, thickness: 1 });
    expected.push(CanvasOp::Outline { area: layout.cell(1), color: rgb(1.0, 1.0, 1.0), alpha: 1.0, thickness: 1 });
    expected.extend([
        item("pickaxe", layout.icon(0)),
        item("bundle", icon),
        item("stone", strip_icon(0)),
        item("pickaxe", strip_icon(1)),
        fill(bar.background(layout.icon(0)), black),
        fill(bar.foreground(layout.icon(0)), bar.color()),
        fill(FillBar::background(icon), black),
        fill(FillBar::foreground(icon, 0.5), rgb(112.0 / 255.0, 134.0 / 255.0, 1.0)),
        fill(bar.background(strip_icon(1)), black),
        fill(bar.foreground(strip_icon(1)), bar.color()),
        CanvasOp::Count { text: "5".to_string(), cell: layout.strip_cell(0).unwrap() },
        CanvasOp::Label { text: "Shift + scroll: next bundle (1/2)".to_string(), area: layout.hint().unwrap() },
        CanvasOp::FlushText,
    ]);
    assert_eq!(canvas.ops, expected);
}

#[test]
fn the_hint_is_one_label_and_one_flush_and_no_hint_draws_no_label() {
    let layout = panel_layout(1, true);
    let (cache, _source, _drops) = bundle_cache();
    let mut hinted = RecordingCanvas::default();
    let mut quiet = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, Some(1), Some("hint"), &mut hinted);
    PreviewRenderer::draw(&panel_layout(1, false), &cache, Some(1), None, &mut quiet);

    assert_eq!(hinted.ops.iter().filter(|op| matches!(op, CanvasOp::Label { .. })).count(), 1);
    assert_eq!(hinted.ops.iter().filter(|op| **op == CanvasOp::FlushText).count(), 1);
    assert!(!quiet.ops.iter().any(|op| matches!(op, CanvasOp::Label { .. })));
}

#[test]
fn an_empty_bundle_and_one_with_no_weight_show_no_fill_bar_and_a_full_one_fills_the_bar() {
    let layout = layout();
    let (cache, _drops) = cache_of(vec![
        FakeEntry::item(Some(0), "bundle").bundle(Some(0), None),
        FakeEntry::item(Some(1), "bundle").bundle(None, None),
        FakeEntry::item(Some(2), "bundle").bundle(Some(64), None),
    ]);
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, None, None, &mut canvas);

    let fills: Vec<&CanvasOp> = canvas.ops[29..].iter().filter(|op| matches!(op, CanvasOp::Fill { .. })).collect();
    assert_eq!(fills.len(), 2);
    assert_eq!(
        *fills[1],
        CanvasOp::Fill {
            area: FillBar::foreground(layout.icon(2), 1.0),
            color: rgb(112.0 / 255.0, 134.0 / 255.0, 1.0),
            alpha: 1.0
        }
    );
}

#[test]
fn an_empty_strip_draws_one_row_of_cells() {
    let layout = panel_layout(1, false);
    let drops = Rc::new(Cell::new(0));
    let source = FakeBox::new(&drops, vec![FakeEntry::item(Some(0), "bundle").bundle(Some(1), Some(vec![]))]);
    let mut cache = PreviewCache::new();
    cache.sync(PreviewKey { stack: 1, user_data: 2, count: 1 }, Some(&source));
    cache.sync_strip(Some(&source), Some(0));
    let mut canvas = RecordingCanvas::default();

    PreviewRenderer::draw(&layout, &cache, Some(0), None, &mut canvas);

    let cell_fills = canvas
        .ops
        .iter()
        .filter(|op| matches!(op, CanvasOp::Fill { color, .. } if *color == rgb(0.23, 0.23, 0.23)))
        .count();
    assert_eq!(cell_fills, 27 + 9);
}
