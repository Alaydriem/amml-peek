use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::Instant;

use amm_client::link::ApiTable;
use amm_core::errors::LinkError;
use amml_game::{EntryId, GameEntries, GameHeap, OwnedItemStack, ScreenView, StopLatch, UiRenderContext, UiScene};
use amml_msvc::MsvcString;
use amml_sdk::{HookHandle, ModRuntime, PanicText};
use peek_core::services::ScrollStep;
use peek_core::structs::Point;

use crate::game::{GameBoxResolver, ShiftKey, UiCanvas};
use crate::services::{ManagerSession, PreviewPanel, RenderTelemetry};
use crate::types::{DrawOutcome, LinkState, PeekFeature, ViewFrame};

/// What the detours share: the stop latch, the entries, the on state and the state of the current enable. The hover
/// hooks and the render hook run on the client thread; disable may run on another thread and waits for the lock.
/// The render hook takes the lock with `try_lock`, so a frame never waits.
pub struct PeekRuntime {
    config_dir: PathBuf,
    latch: StopLatch,
    entries: OnceLock<GameEntries>,
    // whether peek is linked to the manager and the user has it turned on; read by the hover hooks and the hover
    // text hook without the lock
    on: AtomicBool,
    started: Instant,
    // whether Shift is down; read inside the wheel hook
    shift: fn() -> bool,
    state: Mutex<RuntimeState>,
}

// everything behind the lock. The stacks in the panel are blocks of the game heap, dropped on the client thread
struct RuntimeState {
    session: Option<ManagerSession>,
    link: LinkState,
    panel: PreviewPanel<OwnedItemStack<'static>>,
    telemetry: RenderTelemetry,
    hooks: Vec<HookHandle>,
    // set by a stop: the panel is cleared by the next hook call, on the client thread, not by the stopping thread
    stale: bool,
}

// SAFETY: the raw pointers in the state are blocks of the game heap and addresses of game objects. They are used by
// the hover hooks, the render hook and the wheel hook, only under the state lock, and nowhere else
#[allow(unsafe_code)]
unsafe impl Send for RuntimeState {}

static RUNTIME: OnceLock<PeekRuntime> = OnceLock::new();

impl PeekRuntime {
    // the detours: render, hover, unhover, controller destructor, hover text, wheel
    pub const HOOK_CAPACITY: usize = 6;

    pub fn new(config_dir: &Path) -> Self {
        Self::with_shift(config_dir, ShiftKey::held)
    }

    /// A runtime that reads the Shift key with `shift`.
    pub fn with_shift(config_dir: &Path, shift: fn() -> bool) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
            latch: StopLatch::new(),
            entries: OnceLock::new(),
            on: AtomicBool::new(false),
            started: Instant::now(),
            shift,
            state: Mutex::new(RuntimeState {
                session: None,
                link: LinkState::Off,
                panel: PreviewPanel::new(),
                telemetry: RenderTelemetry::new(),
                hooks: Vec::with_capacity(Self::HOOK_CAPACITY),
                stale: false,
            }),
        }
    }

    /// The runtime of this DLL; the first call wins.
    pub fn install(config_dir: &Path) -> &'static PeekRuntime {
        RUNTIME.get_or_init(|| Self::new(config_dir))
    }

    /// As `install`, with the Shift key read by `shift`.
    pub fn install_with_shift(config_dir: &Path, shift: fn() -> bool) -> &'static PeekRuntime {
        RUNTIME.get_or_init(|| Self::with_shift(config_dir, shift))
    }

    pub fn get() -> Option<&'static PeekRuntime> {
        RUNTIME.get()
    }

    /// Raised by disable and by a panic in a hook body.
    pub fn latch(&self) -> &StopLatch {
        &self.latch
    }

    /// Whether peek is linked to the manager and turned on.
    pub fn is_on(&self) -> bool {
        self.on.load(Ordering::SeqCst)
    }

    /// Whether the pointer is over a slot of the screen of `controller`.
    pub fn is_tracking(&self, controller: usize) -> bool {
        self.lock().panel.is_tracking(controller)
    }

    fn lock(&self) -> MutexGuard<'_, RuntimeState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Enable: keeps the entries and reaches the manager now; when the manager is not there yet, the first render
    /// call tries again. A lock poisoned by a panic before the last disable is cleared.
    pub fn start(&self, entries: GameEntries) {
        let scroll = PeekFeature::is_on(&entries, PeekFeature::SCROLL);
        let _ = self.entries.set(entries);
        if self.state.is_poisoned() {
            self.state.clear_poison();
        }
        let mut state = self.lock();
        state.panel.set_scroll(scroll);
        state.link = LinkState::Waiting;
        self.latch.lower();
        self.connect(&mut state, false);
    }

    /// Takes a registration made elsewhere as the link to the manager. The manager has queued a disable first when
    /// the user turned peek off earlier, so the events are read at once.
    pub fn attach(&self, session: ManagerSession) {
        let mut state = self.lock();
        self.adopt(&mut state, session);
    }

    /// Keeps an installed hook; false, keeping nothing, when `HOOK_CAPACITY` hooks are already kept.
    pub fn keep_hook(&self, hook: HookHandle) -> bool {
        let mut state = self.lock();
        if state.hooks.len() >= Self::HOOK_CAPACITY {
            return false;
        }
        state.hooks.push(hook);
        true
    }

    /// The installed hooks, newest first, leaving none behind.
    pub fn take_hooks(&self) -> Vec<HookHandle> {
        let mut hooks = std::mem::take(&mut self.lock().hooks);
        hooks.reverse();
        hooks
    }

    /// Disable, after the hooks are gone: the registration ends and the panel is cleared by the next enable's first
    /// hook call.
    pub fn stop(&self) {
        self.latch.raise();
        self.on.store(false, Ordering::SeqCst);
        let mut state = self.lock();
        state.session = None;
        state.link = LinkState::Off;
        state.stale = true;
    }

    /// After `_onContainerSlotHovered`. `collection` is the address of the `std::string` argument of the call.
    pub fn on_hover(&self, controller: usize, collection: usize, slot: i32) {
        self.guarded("the hover hook", || {
            if !self.is_on() {
                return;
            }
            let mut state = self.lock();
            Self::freshen(&mut state);
            if let Some(name) = Self::collection_name(collection) {
                state.panel.hovered(controller, name, slot);
            }
        });
    }

    /// After `_onContainerSlotUnhovered`.
    pub fn on_unhover(&self, controller: usize, collection: usize, slot: i32) {
        self.guarded("the unhover hook", || {
            if !self.is_on() {
                return;
            }
            let mut state = self.lock();
            Self::freshen(&mut state);
            if let Some(name) = Self::collection_name(collection) {
                state.panel.unhovered(controller, name, slot);
            }
        });
    }

    /// Before `ScreenController::~ScreenController`: no later read goes through the controller.
    pub fn on_controller_dtor(&self, controller: usize) {
        self.guarded("the controller hook", || {
            self.lock().panel.left(controller);
        });
    }

    /// After `ScreenView::render`: reads the manager's events, then, when the view is a container screen with the
    /// pointer over a shulker box, draws the panel. `view` is the `ScreenView` and `ctx` the UI render context of the
    /// running call.
    #[allow(unsafe_code)]
    pub fn on_render(&'static self, view: usize, ctx: usize) {
        self.guarded("the render hook", || {
            let Some(entries) = self.entries.get() else { return };
            // the hover hooks may hold the lock; this frame is then skipped
            let Ok(mut state) = self.state.try_lock() else { return };
            if state.link == LinkState::Waiting {
                self.connect(&mut state, true);
            }
            self.drain(&mut state);
            Self::freshen(&mut state);
            if !self.is_on() {
                return;
            }
            // SAFETY: `view` is the `this` of the running render call.
            let Some(view) = (unsafe { ScreenView::from_raw(view as *mut c_void, entries) }) else { return };
            // SAFETY: `ctx` is the UIRenderContext argument of the running render call.
            let Some(ui) = (unsafe { UiRenderContext::from_raw(ctx as *mut c_void, entries) }) else { return };
            let Some(container) = view.controller().and_then(|controller| controller.container()) else { return };
            let (Some(pointer), Some(size)) = (view.pointer(), view.size()) else { return };
            let Some(heap) = GameHeap::msvc() else { return };
            let frame = ViewFrame {
                controller: container.address(),
                pointer: Point { x: pointer[0], y: pointer[1] },
                screen: Point { x: size[0], y: size[1] },
            };
            let started = Instant::now();
            let resolver = GameBoxResolver::new(&container, heap);
            let mut canvas = UiCanvas::new(&ui, entries);
            let outcome = state.panel.draw(&frame, &resolver, &mut canvas);
            drop(canvas);
            if outcome == DrawOutcome::Drawn {
                let micros = started.elapsed().as_secs_f32() * 1_000_000.0;
                if let Some(report) = state.telemetry.record(micros, self.started.elapsed().as_secs_f64()) {
                    curia::info!("preview draw time", {
                        count: u64::from(report.count),
                        mean_us: f64::from(report.mean_us),
                        p99_us: f64::from(report.p99_us),
                        max_us: f64::from(report.max_us),
                    });
                }
            }
        });
    }

    /// Before `UIScene::handleDirection`: true when peek takes the notch and the game must not see it. `scene` is the
    /// `UIScene`, `y` the step of the notch. A busy lock, a missing entry or a panic give false, so the notch goes
    /// on to the game.
    #[allow(unsafe_code)]
    pub fn on_scroll(&self, scene: usize, direction: u8, _x: f32, y: f32) -> bool {
        self.guarded_or("the scroll hook", false, || {
            let notches = ScrollStep::notches(y);
            if direction != UiScene::DIRECTION_SCROLL || notches == 0 || !self.is_on() {
                return false;
            }
            let Some(entries) = self.entries.get() else { return false };
            // the render hook may hold the lock; the notch then goes to the game
            let Ok(mut state) = self.state.try_lock() else { return false };
            Self::freshen(&mut state);
            // SAFETY: `scene` is the `this` of the running handleDirection call.
            let controller = unsafe { UiScene::from_raw(scene as *mut c_void, entries) }
                .and_then(|scene| scene.screen_view())
                .and_then(|view| view.controller())
                .map(|controller| controller.address());
            state.panel.scroll(controller, (self.shift)(), notches)
        })
    }

    /// The function the hover text detour calls: the base `Item::appendFormattedHovertext` while peek is on, so the
    /// shulker box override does not append its contents lines; `original` otherwise.
    pub fn hovertext_target(&self, original: usize) -> usize {
        if self.latch.raised() || ModRuntime::is_disabled() || !self.is_on() {
            return original;
        }
        self.entries
            .get()
            .and_then(|entries| entries.get(EntryId::ITEM_APPEND_FORMATTED_HOVERTEXT))
            .and_then(|base| usize::try_from(base).ok())
            .filter(|base| *base != 0)
            .unwrap_or(original)
    }

    // the std::string argument of a hover call, as text
    #[allow(unsafe_code)]
    fn collection_name<'a>(collection: usize) -> Option<&'a str> {
        // SAFETY: `collection` is the address of the std::string reference argument of the running hover call, or 0.
        let collection = unsafe { (collection as *const MsvcString).as_ref() }?;
        collection.to_str().ok()
    }

    // drops what a stop left behind
    fn freshen(state: &mut RuntimeState) {
        if state.stale {
            state.panel.clear();
            state.stale = false;
        }
    }

    // reads the manager's events: an off state, and any disable, clears the panel
    fn drain(&self, state: &mut RuntimeState) {
        let Some(session) = state.session.as_mut() else { return };
        let delta = session.drain();
        state.panel.apply(delta);
        if let Some(on) = delta.on {
            self.on.store(on, Ordering::SeqCst);
        }
    }

    fn adopt(&self, state: &mut RuntimeState, session: ManagerSession) {
        state.session = Some(session);
        state.link = LinkState::Attached;
        self.on.store(true, Ordering::SeqCst);
        self.drain(state);
    }

    fn connect(&self, state: &mut RuntimeState, last_try: bool) {
        match ApiTable::resolve().and_then(|table| ManagerSession::connect(table, &self.config_dir)) {
            Ok(session) => self.adopt(state, session),
            Err(error) if last_try => {
                Self::log_link_error(&error);
                state.link = LinkState::Off;
            }
            Err(_) => {}
        }
    }

    // runs `body` unless the mod is stopped; a panic is logged once and raises the latch, so the hooks stop and the
    // game goes on
    fn guarded(&self, what: &str, body: impl FnOnce()) {
        self.guarded_or(what, (), body);
    }

    // as `guarded`, for a body that gives a value; `fallback` is the value when the body did not run or panicked
    fn guarded_or<T>(&self, what: &str, fallback: T, body: impl FnOnce() -> T) -> T {
        if self.latch.raised() || ModRuntime::is_disabled() {
            return fallback;
        }
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(value) => value,
            Err(payload) => {
                self.latch.raise();
                self.on.store(false, Ordering::SeqCst);
                if let Some(runtime) = ModRuntime::get() {
                    runtime
                        .logger()
                        .error(&format!("{what} panicked: {}; the preview stops", PanicText::of(payload.as_ref())));
                }
                fallback
            }
        }
    }

    fn log_link_error(error: &LinkError) {
        let Some(runtime) = ModRuntime::get() else { return };
        match error {
            LinkError::ManagerMissing | LinkError::AbiMismatch { .. } => {
                runtime.logger().error("@alaydriem/peek needs @amml/modmanager; the preview stays off");
            }
            other => {
                runtime.logger().error(&format!("@alaydriem/peek could not start ({other}); the preview stays off"));
            }
        }
    }
}
