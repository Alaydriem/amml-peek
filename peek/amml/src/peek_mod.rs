use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use amml_game::{EntryId, EntryNeed, EntryResolver, GameEntries, GameError, HookOrder};
use amml_sdk::amml_abi::AmmlLogLevel;
use amml_sdk::{AmmlMod, Api, HookHandle, ModDirs, SdkError};

use crate::detours::{screen_controller_dtor, screen_render, scroll, shulker_hovertext, slot_hovered, slot_unhovered};
use crate::services::PeekRuntime;
use crate::types::PeekFeature;

// the AMML entry point: load notes the folders, enable resolves the entries, installs the hooks and starts the
// runtime, disable removes the hooks and stops it
pub struct PeekMod;

impl PeekMod {
    // every entry the mod asks for, each under the feature of its group
    pub fn wanted() -> Vec<(EntryId, EntryNeed)> {
        let required = PeekFeature::REQUIRED.iter().copied();
        required
            .chain([PeekFeature::FONT, PeekFeature::SCROLL])
            .flat_map(|feature| {
                PeekFeature::entries(feature).iter().map(move |id| (*id, EntryNeed::Optional { feature }))
            })
            .collect()
    }

    // the entries; an entry that does not resolve only turns its feature off
    pub fn resolve<R: EntryResolver>(resolver: &R) -> Result<GameEntries, GameError> {
        GameEntries::resolve(resolver, &Self::wanted())
    }

    // the entries of the four required features that did not resolve
    pub fn missing(entries: &GameEntries) -> Vec<&'static str> {
        PeekFeature::REQUIRED
            .iter()
            .flat_map(|feature| PeekFeature::entries(feature))
            .filter(|id| entries.get(**id).is_none())
            .map(|id| id.as_str())
            .collect()
    }

    // a line for each optional feature that is off, naming the entry or the mirror that turned it off and what is lost
    pub fn off_lines(entries: &GameEntries) -> Vec<String> {
        let off = entries.off_features();
        [
            (PeekFeature::FONT, "stack counts are not drawn"),
            (PeekFeature::SCROLL, "the bundle selection by wheel is not available"),
        ]
        .into_iter()
        .filter_map(|(feature, lost)| {
            let cause = off
                .iter()
                .find(|(off_feature, _)| *off_feature == feature)
                .map(|(_, cause)| cause.clone())
                .or_else(|| PeekFeature::misfit(entries, feature))?;
            Some(format!("{feature} is off ({cause}); {lost}"))
        })
        .collect()
    }

    #[allow(unsafe_code)]
    fn hook(
        api: &Api,
        runtime: &PeekRuntime,
        target: u64,
        priority: i32,
        detour: *const c_void,
        original: &'static AtomicPtr<c_void>,
    ) -> Result<(), SdkError> {
        // SAFETY: `target` is the derived start of the game function each caller pairs with the detour written for
        // its signature (this in rcx, then the C++ parameters as the game passes them); each original slot belongs
        // to one hook only
        let hook = unsafe { api.hook(target, detour, priority, original) }?;
        if runtime.keep_hook(hook) {
            return Ok(());
        }
        let _ = api.unhook(hook);
        Err(SdkError::Mod(format!("more than {} hooks", PeekRuntime::HOOK_CAPACITY)))
    }

    fn remove_hooks(api: &Api, runtime: &PeekRuntime) {
        let hooks: Vec<HookHandle> = runtime.take_hooks();
        for hook in hooks {
            if let Err(error) = api.unhook(hook) {
                api.log(&Self::id(), AmmlLogLevel::WARN, &format!("unhook failed: {error}"));
            }
        }
    }

    // the render and hover hooks only read after the game, so they run after every writer; the hover text hook
    // changes what the game builds and the wheel hook may take a notch from it, so both are writers
    fn install_hooks(api: &Api, runtime: &PeekRuntime, entries: &GameEntries) -> Result<(), SdkError> {
        let target = |id: EntryId| entries.get(id).ok_or_else(|| SdkError::MissingEntry(id.as_str().to_string()));
        let read = HookOrder::READ_AFTER_WRITERS;
        Self::hook(
            api,
            runtime,
            target(EntryId::SCREEN_VIEW_RENDER)?,
            read,
            screen_render::detour as *const c_void,
            screen_render::GUARD.original(),
        )?;
        Self::hook(
            api,
            runtime,
            target(EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_HOVERED)?,
            read,
            slot_hovered::detour as *const c_void,
            slot_hovered::GUARD.original(),
        )?;
        Self::hook(
            api,
            runtime,
            target(EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_UNHOVERED)?,
            read,
            slot_unhovered::detour as *const c_void,
            slot_unhovered::GUARD.original(),
        )?;
        Self::hook(
            api,
            runtime,
            target(EntryId::SCREEN_CONTROLLER_DTOR)?,
            read,
            screen_controller_dtor::detour as *const c_void,
            screen_controller_dtor::GUARD.original(),
        )?;
        Self::hook(
            api,
            runtime,
            target(EntryId::SHULKER_BOX_BLOCK_ITEM_APPEND_FORMATTED_HOVERTEXT)?,
            HookOrder::WRITE,
            shulker_hovertext::detour as *const c_void,
            shulker_hovertext::GUARD.original(),
        )?;
        if !PeekFeature::is_on(entries, PeekFeature::SCROLL) {
            return Ok(());
        }
        Self::hook(
            api,
            runtime,
            target(EntryId::UI_SCENE_HANDLE_DIRECTION)?,
            HookOrder::WRITE,
            scroll::detour as *const c_void,
            scroll::GUARD.original(),
        )
    }
}

impl AmmlMod for PeekMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "peek";

    // runs under the loader lock: only the folders are noted, nothing starts a thread
    fn load(api: &Api) -> Result<(), SdkError> {
        let dirs = ModDirs::new(api, &Self::id())?;
        PeekRuntime::install(&dirs.config_dir);
        Ok(())
    }

    // with an entry of the four game features missing (a build the entries are not for) nothing is installed
    fn enable(api: &Api) -> Result<(), SdkError> {
        let runtime = PeekRuntime::get().ok_or_else(|| SdkError::Mod("load did not run".to_string()))?;
        let entries = Self::resolve(api).map_err(|error| SdkError::Mod(error.to_string()))?;
        let missing = Self::missing(&entries);
        if !missing.is_empty() {
            let line = format!("peek is off, game entries missing: {}", missing.join(", "));
            api.log(&Self::id(), AmmlLogLevel::WARN, &line);
            return Ok(());
        }
        let off_lines = Self::off_lines(&entries);
        for line in &off_lines {
            api.log(&Self::id(), AmmlLogLevel::WARN, line);
        }
        if off_lines.is_empty() {
            api.log(&Self::id(), AmmlLogLevel::INFO, "features off: none");
        }
        if let Err(error) = Self::install_hooks(api, runtime, &entries) {
            Self::remove_hooks(api, runtime);
            return Err(error);
        }
        runtime.start(entries);
        Ok(())
    }

    // the hooks go first, so no new call enters the runtime; then the registration ends
    fn disable(api: &Api) {
        let Some(runtime) = PeekRuntime::get() else { return };
        runtime.latch().raise();
        Self::remove_hooks(api, runtime);
        runtime.stop();
    }
}
