use std::path::Path;

use alaydriem_peek::game::ShiftKey;
use alaydriem_peek::services::{ManagerSession, PeekRuntime};
use amm_client::test_support::FakeTable;
use amml_game::EntryId;
use amml_msvc::MsvcString;

use super::{FakeResolver, TestHeap};

// starts the runtime of this test process on a fake manager. Each test runs in its own process, so the
// singleton is fresh
pub struct RuntimeHarness;

impl RuntimeHarness {
    pub fn start(values: &[(EntryId, u64)]) -> &'static PeekRuntime {
        Self::start_with_shift(values, ShiftKey::held)
    }

    // as `start`, with a Shift key that answers with `shift`
    pub fn start_with_shift(values: &[(EntryId, u64)], shift: fn() -> bool) -> &'static PeekRuntime {
        let runtime = PeekRuntime::install_with_shift(Path::new(FakeTable::CONFIG_DIR), shift);
        runtime.start(FakeResolver::entries(values));
        let session = ManagerSession::connect(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();
        runtime.attach(session);
        runtime
    }

    // the address of a game string that is never freed, as a hover call passes it
    pub fn collection(text: &str) -> usize {
        // SAFETY: the test heap is the Rust allocator
        let name = unsafe { MsvcString::new_in(&TestHeap::heap(), text) }.unwrap();
        Box::into_raw(Box::new(name)) as usize
    }
}
