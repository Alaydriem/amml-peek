use std::sync::{Arc, Mutex};

use amml_api::HookOrder;
use amml_api::amml_abi::ScreenRender;
use amml_api::testing::{FakeGame, FakeScreen};

use super::peek_rig::{SCREEN, VIEW};

// runs a closure once inside the `ScreenRender` dispatch of a fake screen, where the frame handles of the event are
// valid, and gives back what it returned
pub struct ScreenProbe;

impl ScreenProbe {
    pub fn run<R, F>(game: &FakeGame, screen: &FakeScreen, body: F) -> R
    where
        R: Send + 'static,
        F: FnOnce(&ScreenRender) -> R + Send + 'static,
    {
        let body = Mutex::new(Some(body));
        let out: Arc<Mutex<Option<R>>> = Arc::new(Mutex::new(None));
        let sink = Arc::clone(&out);
        let listener = game
            .ctx()
            .events()
            .listen::<ScreenRender>(HookOrder::NORMAL, move |event, _| {
                if let Some(body) = body.lock().unwrap().take() {
                    *sink.lock().unwrap() = Some(body(event));
                }
            })
            .unwrap();
        game.raise_screen_render(SCREEN, VIEW, screen);
        drop(listener);
        let result = out.lock().unwrap().take();
        result.unwrap()
    }
}
