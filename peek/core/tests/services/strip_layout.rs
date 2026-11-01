use peek_core::services::StripLayout;

#[test]
fn rows_grow_by_nine_entries_and_stop_at_eight() {
    let rows: Vec<usize> = [0, 1, 9, 10, 64, 100].iter().map(|entries| StripLayout::rows(*entries)).collect();

    assert_eq!(rows, [1, 1, 1, 2, 8, 8]);
}

#[test]
fn cells_are_row_major_and_end_at_64() {
    assert_eq!(StripLayout::cell(0), Some((0, 0)));
    assert_eq!(StripLayout::cell(10), Some((1, 1)));
    assert_eq!(StripLayout::cell(63), Some((0, 7)));
    assert_eq!(StripLayout::cell(64), None);
}

fn ordered(slots: &[Option<i32>]) -> Vec<usize> {
    let mut out = [0usize; StripLayout::MAX_ENTRIES];
    let count = StripLayout::order(slots.len(), |index| slots[index], &mut out);
    out[..count].to_vec()
}

#[test]
fn entries_come_in_slot_order() {
    assert_eq!(ordered(&[Some(5), Some(0), Some(3)]), [1, 2, 0]);
}

#[test]
fn entries_with_a_bad_slot_keep_list_order_after_the_good_ones() {
    assert_eq!(ordered(&[None, Some(2), Some(-4), Some(1), None]), [3, 1, 0, 2, 4]);
}

#[test]
fn only_the_first_64_entries_are_ordered() {
    let slots: Vec<Option<i32>> = (0..100).map(|index| Some(100 - index)).collect();

    let order = ordered(&slots);

    assert_eq!(order.len(), 64);
    assert_eq!(order[0], 63);
    assert!(order.iter().all(|index| *index < 64));
}
