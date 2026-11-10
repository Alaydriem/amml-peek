use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use alaydriem_peek::detours::{scroll, slot_hovered};
use amml_game::EntryId;

use crate::support::RuntimeHarness;

const SCROLL: u8 = 2;

// the scene, the direction, x, y and the focus impact the original saw
type Call = (usize, u8, f32, f32, u8);

static SEEN: Mutex<Vec<Call>> = Mutex::new(Vec::new());

extern "C" fn game_scroll(scene: usize, direction: u8, x: f32, y: f32, focus_impact: u8) {
    SEEN.lock().unwrap().push((scene, direction, x, y, focus_impact));
}

extern "C" fn game_hovered(_: usize, _: usize, _: i32) -> u32 {
    0
}

fn shift() -> bool {
    true
}

// a UIScene whose view pointer sits at 0x40, and a ScreenView whose controller sits at 0x40
struct FakeScene {
    scene: Box<[usize; 16]>,
    _view: Box<[usize; 10]>,
}

impl FakeScene {
    fn new(controller: usize) -> Self {
        let mut view = Box::new([0usize; 10]);
        view[8] = controller;
        let mut scene = Box::new([0usize; 16]);
        scene[8] = view.as_ptr() as usize;
        Self { scene, _view: view }
    }

    fn address(&self) -> usize {
        self.scene.as_ptr() as usize
    }
}

fn started() -> &'static alaydriem_peek::services::PeekRuntime {
    RuntimeHarness::start_with_shift(
        &[(EntryId::UI_SCENE_SCREEN_VIEW, 0x40), (EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS, 1)],
        shift,
    )
}

#[test]
fn without_a_runtime_the_original_runs_with_the_same_five_arguments() {
    scroll::GUARD.original().store(game_scroll as *mut c_void, Ordering::SeqCst);

    scroll::detour(0x1000, SCROLL, 0.0, -0.94, 1);

    assert_eq!(SEEN.lock().unwrap().clone(), [(0x1000, SCROLL, 0.0, -0.94, 1)]);
}

#[test]
fn a_notch_with_nothing_hovered_goes_on_to_the_game_unchanged() {
    started();
    let scene = FakeScene::new(0x10);
    scroll::GUARD.original().store(game_scroll as *mut c_void, Ordering::SeqCst);

    scroll::detour(scene.address(), SCROLL, 0.0, -0.94, 0);

    assert_eq!(SEEN.lock().unwrap().clone(), [(scene.address(), SCROLL, 0.0, -0.94, 0)]);
}

#[test]
fn a_notch_on_the_hovered_screen_without_two_bundles_goes_on_to_the_game() {
    started();
    let collection = RuntimeHarness::collection("container_items");
    let scene = FakeScene::new(0x10);
    scroll::GUARD.original().store(game_scroll as *mut c_void, Ordering::SeqCst);
    slot_hovered::GUARD.original().store(game_hovered as *mut c_void, Ordering::SeqCst);
    slot_hovered::detour(0x10, collection, 3);

    scroll::detour(scene.address(), SCROLL, 0.0, 0.94, 0);

    assert_eq!(SEEN.lock().unwrap().clone(), [(scene.address(), SCROLL, 0.0, 0.94, 0)]);
}

#[test]
fn a_direction_that_is_not_a_scroll_goes_on_to_the_game() {
    started();
    let scene = FakeScene::new(0x10);
    scroll::GUARD.original().store(game_scroll as *mut c_void, Ordering::SeqCst);

    scroll::detour(scene.address(), 1, 0.0, -0.94, 0);

    assert_eq!(SEEN.lock().unwrap().clone(), [(scene.address(), 1, 0.0, -0.94, 0)]);
}

#[test]
fn a_scene_that_is_not_readable_goes_on_to_the_game() {
    started();
    scroll::GUARD.original().store(game_scroll as *mut c_void, Ordering::SeqCst);

    scroll::detour(0, SCROLL, 0.0, -0.94, 0);

    assert_eq!(SEEN.lock().unwrap().clone(), [(0, SCROLL, 0.0, -0.94, 0)]);
}
