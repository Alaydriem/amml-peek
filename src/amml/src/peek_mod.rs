use std::sync::{Arc, Mutex, PoisonError};

use amml_api::amml_abi::{
    ContainerSlotHover, ContainerSlotUnhover, InjectedInputReleased, InputLeaseLost, InputLeaseResumed,
    InputLeaseSuspended, ItemHovertext, KeyInput, ModsLoaded, ProviderGone, ProviderReady, ScreenClosing, ScreenRender,
    ScreenshotTaken, ScreenshotWritten, UiScroll,
};
use amml_api::{AmmlMod, ApiError, Area, Ctx, DisableCtx, Uses};

use crate::services::{PeekLink, PeekListeners, PeekRuntime};

// what the enabled mod owns. `disable` takes it, which drops the listeners and closes the link
static RUNNING: Mutex<Option<PeekListeners>> = Mutex::new(None);

// the AMML entry point: enable starts the manager link and the listeners, disable removes them and leaves the
// mod manager
pub struct PeekMod;

impl AmmlMod for PeekMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "peek";
    const USES: Uses = amml_api::amml_uses! {
        feature "preview" {
            required ScreenRender;
            required ContainerSlotHover;
            required ContainerSlotUnhover;
            required ScreenClosing;
            required ProviderReady;
            required ProviderGone;
            required ModsLoaded;
            required area Area::UI_DRAW.at(2);
            required area Area::UI_SCREEN.at(2);
            required area Area::INVENTORY_READ.at(4);
            // The link listens to these when the loader serves them
            optional InputLeaseSuspended;
            optional InputLeaseResumed;
            optional InputLeaseLost;
            optional InjectedInputReleased;
            optional ScreenshotTaken;
            optional ScreenshotWritten;
            optional area Area::INPUT_LEASE;
        }
        feature "counts" {
            required area Area::UI_TEXT;
        }
        feature "bundle_pick" {
            required UiScroll;
            required KeyInput;
        }
        feature "hovertext" {
            required ItemHovertext;
        }
    };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let log = ctx.log();
        let link = PeekLink::start(ctx).map_err(|error| {
            log.error("peek off", &[("error", &error.to_string())]);
            ApiError::Refused
        })?;
        let runtime = Arc::new(PeekRuntime::new(log));
        runtime.attach(link);
        let listeners = PeekListeners::start(ctx, Arc::clone(&runtime)).inspect_err(|error| {
            log.error("peek off", &[("error", &error.to_string())]);
            runtime.detach();
        })?;
        log.info("features off: none", &[]);
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(listeners);
        Ok(())
    }

    fn disable(_ctx: &Ctx, _how: &DisableCtx) {
        let running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(listeners) = running {
            listeners.stop();
        }
    }
}
