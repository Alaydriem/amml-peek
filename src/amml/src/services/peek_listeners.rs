use std::sync::Arc;

use amml_api::amml_abi::{
    ContainerSlotHover, ContainerSlotUnhover, ItemHovertext, ModsLoaded, ScreenClosing, ScreenRender, UiScroll,
};
use amml_api::{
    ApiError, Cancel, ContainerSlotHoverExt, Ctx, HookOrder, HovertextKind, ItemHovertextExt, Listener,
    ScreenClosingExt, UiScrollExt,
};

use crate::services::PeekRuntime;

/// The loader listeners of the preview; each only hands its event to the runtime. Dropping them removes them.
pub struct PeekListeners {
    listeners: Vec<Listener>,
    runtime: Arc<PeekRuntime>,
}

impl PeekListeners {
    /// Every listener is required: a refused listen fails the start and drops the ones made so far.
    pub fn start(ctx: &Ctx, runtime: Arc<PeekRuntime>) -> Result<Self, ApiError> {
        let events = ctx.events();
        let mut listeners = Vec::with_capacity(7);
        let drawing = Arc::clone(&runtime);
        listeners.push(events.listen::<ScreenRender>(HookOrder::READ_AFTER_WRITERS, move |event, _| {
            drawing.on_render(event);
        })?);
        let hovering = Arc::clone(&runtime);
        listeners.push(events.listen::<ContainerSlotHover>(HookOrder::READ_AFTER_WRITERS, move |event, _| {
            hovering.on_hover(event.screen(), event.slot());
        })?);
        let leaving = Arc::clone(&runtime);
        listeners.push(events.listen::<ContainerSlotUnhover>(HookOrder::READ_AFTER_WRITERS, move |event, _| {
            leaving.on_unhover(event.screen(), event.slot());
        })?);
        let closing = Arc::clone(&runtime);
        listeners.push(events.listen::<ScreenClosing>(HookOrder::READ_AFTER_WRITERS, move |event, _| {
            closing.on_closing(event.screen());
        })?);
        let loaded = Arc::clone(&runtime);
        listeners.push(events.listen::<ModsLoaded>(HookOrder::READ_AFTER_WRITERS, move |_, _| {
            loaded.on_mods_loaded();
        })?);
        let scrolling = Arc::clone(&runtime);
        let input = ctx.input();
        listeners.push(events.listen::<UiScroll>(HookOrder::WRITE, move |event, _| {
            if scrolling.on_scroll(event.screen(), event.notches(), input.modifiers().shift()) {
                event.cancel();
            }
        })?);
        let text = Arc::clone(&runtime);
        listeners.push(events.listen::<ItemHovertext>(HookOrder::WRITE, move |event, _| {
            if event.kind() == HovertextKind::ContainerItem && text.hides_contents() {
                event.hide_contents();
            }
        })?);
        Ok(Self { listeners, runtime })
    }

    /// The listeners go first, so no listener runs while the runtime lets go of the link.
    pub fn stop(self) {
        let Self { listeners, runtime } = self;
        drop(listeners);
        runtime.detach();
    }
}
