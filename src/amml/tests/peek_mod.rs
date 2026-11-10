use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::Mutex;

use alaydriem_peek::PeekMod;
use alaydriem_peek::types::{PeekEntry, PeekFeature};
use amml_game::{ContainerScreenController, EntryId, EntryNeed, UiRenderContext};
use amml_sdk::amml_abi::{AMML_API_VERSION, AmmlApi, AmmlLogLevel, AmmlStatus, AmmlStr};
use amml_sdk::{AmmlMod, Api};

// the loader table is fake: `resolve` answers from `VALUES`, `hook` and `unhook` record their calls
static VALUES: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);
static HOOKS: Mutex<Vec<(u64, i32)>> = Mutex::new(Vec::new());
static UNHOOKED: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static REFUSED: Mutex<Option<u64>> = Mutex::new(None);
static LOGGED: Mutex<Vec<String>> = Mutex::new(Vec::new());
static DIR: &str = "C:/peek-test";

struct FakeLoader;

impl FakeLoader {
    unsafe extern "C" fn resolve(id: AmmlStr, out: *mut u64) -> AmmlStatus {
        // SAFETY: the SDK passes a live string view and a live u64
        let id = unsafe { id.as_str() }.unwrap().to_string();
        match VALUES.lock().unwrap().as_ref().and_then(|values| values.get(&id)) {
            Some(value) => {
                // SAFETY: as above
                unsafe { out.write(*value) };
                AmmlStatus::OK
            }
            None => AmmlStatus::MISSING_ENTRY,
        }
    }

    unsafe extern "C" fn hook(
        target: *const c_void,
        _detour: *const c_void,
        priority: i32,
        _original: *mut *const c_void,
        handle: *mut u64,
    ) -> AmmlStatus {
        if *REFUSED.lock().unwrap() == Some(target as u64) {
            return AmmlStatus::HOOK_FAILED;
        }
        let mut hooks = HOOKS.lock().unwrap();
        hooks.push((target as u64, priority));
        // SAFETY: the SDK passes a live u64
        unsafe { handle.write(hooks.len() as u64) };
        AmmlStatus::OK
    }

    unsafe extern "C" fn unhook(handle: u64) -> AmmlStatus {
        UNHOOKED.lock().unwrap().push(handle);
        AmmlStatus::OK
    }

    unsafe extern "C" fn log(_: AmmlStr, _: AmmlLogLevel, line: AmmlStr) {
        // SAFETY: the SDK passes a live string view
        let line = unsafe { line.as_str() }.unwrap().to_string();
        LOGGED.lock().unwrap().push(line);
    }

    unsafe extern "C" fn heap_new(_: usize) -> *mut u8 {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn heap_delete(_: *mut u8, _: usize) {}

    unsafe extern "C" fn mod_dir(_: AmmlStr, _: u8, out: *mut AmmlStr) -> AmmlStatus {
        // SAFETY: the SDK passes a live view to write; the text is a static
        unsafe { out.write(AmmlStr::from_str(DIR)) };
        AmmlStatus::OK
    }

    fn api() -> Api {
        let table = AmmlApi {
            size: AmmlApi::SIZE_V1,
            version: AMML_API_VERSION,
            module_base: 0x1000,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
        };
        // SAFETY: the table is live for the call, and complete
        unsafe { Api::from_raw(&table, AMML_API_VERSION) }.unwrap()
    }

    // every wanted entry gets its own value, except the ids in `missing`; the font handle size is the mirror size
    fn install(missing: &[EntryId]) {
        let mut values: HashMap<String, u64> = HashMap::new();
        for (index, (id, _)) in PeekMod::wanted().iter().enumerate() {
            if !missing.contains(id) {
                values.insert(id.as_str().to_string(), 0x1000 + 0x10 * index as u64);
            }
        }
        values.insert(PeekEntry::FONT_HANDLE_SIZE.as_str().to_string(), 0x50);
        *VALUES.lock().unwrap() = Some(values);
    }

    fn value(id: EntryId) -> u64 {
        VALUES.lock().unwrap().as_ref().and_then(|values| values.get(id.as_str()).copied()).unwrap()
    }

    fn lines_with(text: &str) -> Vec<String> {
        LOGGED.lock().unwrap().iter().filter(|line| line.contains(text)).cloned().collect()
    }
}

#[test]
fn the_mod_id_is_the_namespaced_id_and_amml_hcl_names_the_same_parts() {
    let document: hcl::Value = hcl::from_str(include_str!("../amml.hcl")).unwrap();
    let hcl::Value::Object(map) = document else { panic!("the document is not an object") };

    assert_eq!(PeekMod::id(), "@alaydriem/peek");
    assert_eq!(map.get("namespace").and_then(hcl::Value::as_str), Some(PeekMod::NAMESPACE));
    assert_eq!(map.get("name").and_then(hcl::Value::as_str), Some(PeekMod::NAME));
}

#[test]
fn amml_hcl_lists_exactly_the_entries_the_mod_asks_for() {
    let document: hcl::Value = hcl::from_str(include_str!("../amml.hcl")).unwrap();
    let listed: HashSet<String> = match document {
        hcl::Value::Object(map) => match map.get("entries") {
            Some(hcl::Value::Array(entries)) => {
                entries.iter().map(|entry| entry.as_str().unwrap().to_string()).collect()
            }
            other => panic!("entries is {other:?}"),
        },
        other => panic!("the document is {other:?}"),
    };
    let wanted: HashSet<String> = PeekMod::wanted().iter().map(|(id, _)| id.as_str().to_string()).collect();

    assert_eq!(listed, wanted);
}

#[test]
fn the_four_game_features_are_required_and_the_font_is_optional_in_its_own_feature() {
    let wanted = PeekMod::wanted();
    let feature_of = |id: EntryId| {
        wanted
            .iter()
            .find_map(|(entry, need)| match (*entry == id, need) {
                (true, EntryNeed::Optional { feature }) => Some(*feature),
                _ => None,
            })
            .unwrap()
    };

    assert_eq!(feature_of(EntryId::SCREEN_VIEW_RENDER), UiRenderContext::FEATURE);
    assert_eq!(feature_of(EntryId::SCREEN_CONTROLLER_DTOR), ContainerScreenController::FEATURE);
    assert_eq!(feature_of(EntryId::ITEM_STACK_FROM_TAG), amml_game::CompoundTagRef::FEATURE);
    assert_eq!(feature_of(EntryId::ITEM_RENDERER_RENDER_GUI_ITEM_NEW), amml_game::ItemRenderer::FEATURE);
    assert_eq!(feature_of(PeekEntry::FONT_GET_LINE_LENGTH), PeekFeature::FONT);
    assert!(!PeekFeature::REQUIRED.contains(&PeekFeature::FONT));
}

#[test]
fn with_every_entry_enable_installs_six_hooks_in_order_and_disable_removes_them_newest_first() {
    FakeLoader::install(&[]);
    let api = FakeLoader::api();

    PeekMod::load(&api).unwrap();
    PeekMod::enable(&api).unwrap();

    let hooks = HOOKS.lock().unwrap().clone();
    let expected = [
        (FakeLoader::value(EntryId::SCREEN_VIEW_RENDER), 100),
        (FakeLoader::value(EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_HOVERED), 100),
        (FakeLoader::value(EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_UNHOVERED), 100),
        (FakeLoader::value(EntryId::SCREEN_CONTROLLER_DTOR), 100),
        (FakeLoader::value(EntryId::SHULKER_BOX_BLOCK_ITEM_APPEND_FORMATTED_HOVERTEXT), 0),
        (FakeLoader::value(EntryId::UI_SCENE_HANDLE_DIRECTION), 0),
    ];
    assert_eq!(hooks, expected);
    assert_eq!(FakeLoader::lines_with("features off: none").len(), 1);

    PeekMod::disable(&api);

    assert_eq!(UNHOOKED.lock().unwrap().clone(), [6, 5, 4, 3, 2, 1]);
}

#[test]
fn a_missing_feature_entry_installs_no_hook_and_logs_one_line_naming_the_missing_entries() {
    FakeLoader::install(&[
        EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_UNHOVERED,
        EntryId::ITEM_STACK_FROM_TAG,
    ]);
    let api = FakeLoader::api();

    PeekMod::load(&api).unwrap();
    assert!(PeekMod::enable(&api).is_ok());

    assert!(HOOKS.lock().unwrap().is_empty());
    let lines = FakeLoader::lines_with("ContainerScreenController.onContainerSlotUnhovered");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("ItemStack.fromTag"), "{lines:?}");
    assert!(FakeLoader::lines_with("features off: none").is_empty());
}

#[test]
fn without_the_font_entries_peek_still_hooks_and_logs_one_line_about_the_counts() {
    FakeLoader::install(&[PeekEntry::FONT_GET_LINE_LENGTH]);
    let api = FakeLoader::api();

    PeekMod::load(&api).unwrap();
    PeekMod::enable(&api).unwrap();

    assert_eq!(HOOKS.lock().unwrap().len(), 6);
    let lines = FakeLoader::lines_with("peek.font is off");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("counts"), "{lines:?}");
}

#[test]
fn a_font_handle_size_that_is_not_the_mirror_size_turns_the_font_off() {
    FakeLoader::install(&[]);
    VALUES.lock().unwrap().as_mut().unwrap().insert(PeekEntry::FONT_HANDLE_SIZE.as_str().to_string(), 0x58);
    let api = FakeLoader::api();

    PeekMod::load(&api).unwrap();
    PeekMod::enable(&api).unwrap();

    let lines = FakeLoader::lines_with("peek.font is off");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("FontHandle.size is 0x58"), "{lines:?}");
}

#[test]
fn a_failed_hook_removes_the_ones_installed_and_fails_the_enable() {
    FakeLoader::install(&[]);
    *REFUSED.lock().unwrap() = Some(FakeLoader::value(EntryId::SCREEN_CONTROLLER_DTOR));
    let api = FakeLoader::api();

    PeekMod::load(&api).unwrap();
    assert!(PeekMod::enable(&api).is_err());

    assert_eq!(HOOKS.lock().unwrap().len(), 3);
    assert_eq!(UNHOOKED.lock().unwrap().clone(), [3, 2, 1]);
}

#[test]
fn without_the_scroll_entries_peek_hooks_five_and_logs_one_line_about_the_bundle_selection() {
    FakeLoader::install(&[EntryId::UI_SCENE_SCREEN_VIEW]);
    let api = FakeLoader::api();

    PeekMod::load(&api).unwrap();
    PeekMod::enable(&api).unwrap();

    let hooks = HOOKS.lock().unwrap().clone();
    assert_eq!(hooks.len(), 5);
    assert!(hooks.iter().all(|(target, _)| *target != FakeLoader::value(EntryId::UI_SCENE_HANDLE_DIRECTION)));
    let lines = FakeLoader::lines_with("game.ui_scroll is off");
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("UIScene.mScreenView"), "{lines:?}");
    assert!(lines[0].contains("bundle selection"), "{lines:?}");
}

#[test]
fn the_scroll_feature_is_optional_and_its_entries_are_the_ui_scene_ones() {
    let wanted = PeekMod::wanted();
    let feature_of = |id: EntryId| {
        wanted
            .iter()
            .find_map(|(entry, need)| match (*entry == id, need) {
                (true, EntryNeed::Optional { feature }) => Some(*feature),
                _ => None,
            })
            .unwrap()
    };

    assert_eq!(feature_of(EntryId::UI_SCENE_HANDLE_DIRECTION), PeekFeature::SCROLL);
    assert_eq!(feature_of(EntryId::UI_SCENE_SCREEN_VIEW), PeekFeature::SCROLL);
    assert!(!PeekFeature::REQUIRED.contains(&PeekFeature::SCROLL));
}
