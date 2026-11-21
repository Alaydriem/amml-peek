use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use amm_client::link::LinkState;
use amml_api::amml_abi::ScreenRender;
use amml_api::{Collection, Logger, ScreenId, ScreenRenderExt, SlotAt};
use peek_core::services::ScrollStep;
use peek_core::structs::{Point, SlotCollection};

use crate::game::{GameBox, GameBoxResolver, UiCanvas};
use crate::services::{PeekLink, PreviewPanel, RenderTelemetry};
use crate::types::{DrawOutcome, ViewFrame};

/// What the listeners share: the log, the on state and the state of the current enable. Every event of the mod runs
/// on the game thread, so the state sits behind one plain lock.
pub struct PeekRuntime {
    log: Logger,
    // whether peek is linked to the manager and the user has it turned on
    on: AtomicBool,
    started: Instant,
    state: Mutex<RuntimeState>,
}

// everything behind the lock
struct RuntimeState {
    link: Option<PeekLink>,
    connected: bool,
    panel: PreviewPanel<GameBox>,
    telemetry: RenderTelemetry,
    // the screen of the last `peek hover` line, and whether the other first-time lines were written
    hover_logged: Option<u64>,
    counts_logged: bool,
    hovertext_logged: bool,
}

impl PeekRuntime {
    pub fn new(log: Logger) -> Self {
        let mut panel = PreviewPanel::new();
        panel.set_scroll(true);
        Self {
            log,
            on: AtomicBool::new(false),
            started: Instant::now(),
            state: Mutex::new(RuntimeState {
                link: None,
                connected: false,
                panel,
                telemetry: RenderTelemetry::new(),
                hover_logged: None,
                counts_logged: false,
                hovertext_logged: false,
            }),
        }
    }

    /// Whether peek is linked to the manager and turned on.
    pub fn is_on(&self) -> bool {
        self.on.load(Ordering::SeqCst)
    }

    fn lock(&self) -> MutexGuard<'_, RuntimeState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Enable: takes the link to the manager. The first render call after it connects turns the preview on.
    pub fn attach(&self, link: PeekLink) {
        let mut state = self.lock();
        state.link = Some(link);
        state.connected = false;
        self.poll(&mut state);
    }

    /// Disable, after the listeners are gone: the registration ends and the panel is cleared.
    pub fn detach(&self) {
        self.on.store(false, Ordering::SeqCst);
        let mut state = self.lock();
        state.link = None;
        state.connected = false;
        state.panel.clear();
        state.hover_logged = None;
    }

    /// Every mod is loaded: a link that is not connected now has no manager to connect to.
    pub fn on_mods_loaded(&self) {
        let state = self.lock();
        let connected = state.link.as_ref().is_some_and(|link| link.state() == LinkState::Connected);
        if !connected {
            self.log.error("@alaydriem/peek needs @amml/modmanager; the preview stays off", &[]);
        }
    }

    /// After the hover handler of the game.
    pub fn on_hover(&self, screen: ScreenId, slot: SlotAt) {
        let mut state = self.lock();
        self.poll(&mut state);
        if !self.is_on() {
            return;
        }
        let collection = Self::collection(slot.collection);
        state.panel.hovered(screen.0, collection, slot.index);
        if state.hover_logged != Some(screen.0) {
            state.hover_logged = Some(screen.0);
            self.log.info(
                "peek hover",
                &[
                    ("screen", &screen.0.to_string()),
                    ("collection", collection.name()),
                    ("slot", &slot.index.to_string()),
                ],
            );
        }
    }

    /// After the unhover handler of the game.
    pub fn on_unhover(&self, screen: ScreenId, slot: SlotAt) {
        if !self.is_on() {
            return;
        }
        self.lock().panel.unhovered(screen.0, Self::collection(slot.collection), slot.index);
    }

    /// Before the game frees the screen: no later read goes through it.
    pub fn on_closing(&self, screen: ScreenId) {
        let mut state = self.lock();
        if state.hover_logged == Some(screen.0) {
            state.hover_logged = None;
            self.log.info("peek screen closed", &[("screen", &screen.0.to_string())]);
        }
        state.panel.left(screen.0);
    }

    /// After the game drew one view: reads the events of the manager, then, when the view is a container screen with
    /// the pointer over a shulker box, draws the panel.
    pub fn on_render(&self, event: &ScreenRender) {
        let mut state = self.lock();
        self.poll(&mut state);
        if !self.is_on() {
            return;
        }
        let (Some(view), Some(container), Some(draw)) = (event.view(), event.container_screen(), event.draw()) else {
            return;
        };
        let (Some(pointer), Some(size)) = (view.pointer(), view.size()) else { return };
        let frame = ViewFrame {
            screen: event.screen().0,
            pointer: Point { x: pointer.x, y: pointer.y },
            size: Point { x: size.width as f32, y: size.height as f32 },
        };
        let started = Instant::now();
        let resolver = GameBoxResolver::new(&container);
        let mut canvas = UiCanvas::new(&draw);
        let outcome = state.panel.draw(&frame, &resolver, &mut canvas);
        let (stale, texts) = (canvas.stale(), canvas.texts());
        if stale {
            state.panel.reload();
        }
        if let Some(summary) = state.panel.take_loaded() {
            self.log
                .info("peek box", &[("items", &summary.items.to_string()), ("bundles", &summary.bundles.to_string())]);
        }
        if texts > 0 && !state.counts_logged {
            state.counts_logged = true;
            self.log.info("peek counts", &[("drawn", &texts.to_string())]);
        }
        if outcome == DrawOutcome::Drawn {
            let micros = started.elapsed().as_secs_f32() * 1_000_000.0;
            if let Some(report) = state.telemetry.record(micros, self.started.elapsed().as_secs_f64()) {
                self.log.info(
                    "preview draw time",
                    &[
                        ("count", &report.count.to_string()),
                        ("mean_us", &format!("{:.1}", report.mean_us)),
                        ("p99_us", &format!("{:.1}", report.p99_us)),
                        ("max_us", &format!("{:.1}", report.max_us)),
                    ],
                );
            }
        }
    }

    /// Before the UI scene handles one wheel step: true when peek takes it and the game must not see it. `notches` is
    /// positive for wheel up. Anything that is not a Shift step over a box with two or more bundles on the hovered
    /// screen goes on to the game.
    pub fn on_scroll(&self, screen: Option<ScreenId>, notches: i32, shift: bool) -> bool {
        let step = ScrollStep::from_notches(notches);
        if step == 0 || !self.is_on() {
            return false;
        }
        let mut state = self.lock();
        let taken = state.panel.scroll(screen.map(|screen| screen.0), shift, step);
        if taken && let Some((index, _)) = state.panel.selected_position() {
            self.log.info("peek bundle selected", &[("index", &index.to_string())]);
        }
        taken
    }

    /// Before the game builds the hover text of a shulker box: true while the preview shows, so the contents lines are
    /// left out.
    pub fn hides_contents(&self) -> bool {
        if !self.is_on() {
            return false;
        }
        let mut state = self.lock();
        if !state.hovertext_logged {
            state.hovertext_logged = true;
            self.log.info("peek hovertext", &[("contents_lines", "0")]);
        }
        true
    }

    // connects on the first call after the manager is there, then reads its events: an off state, and any disable,
    // clears the panel
    fn poll(&self, state: &mut RuntimeState) {
        let Some(link) = state.link.as_mut() else { return };
        if !state.connected && link.state() == LinkState::Connected {
            state.connected = true;
            self.on.store(true, Ordering::SeqCst);
            self.log.info("peek linked", &[]);
        }
        let delta = link.drain();
        state.panel.apply(delta);
        if let Some(on) = delta.on {
            self.on.store(on, Ordering::SeqCst);
        }
    }

    fn collection(collection: Collection) -> SlotCollection {
        match collection {
            Collection::Container => SlotCollection::Container,
            Collection::Hotbar => SlotCollection::Hotbar,
            Collection::Inventory => SlotCollection::Inventory,
            _ => SlotCollection::Other,
        }
    }
}
