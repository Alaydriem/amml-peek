use std::alloc::Layout;

use amml_msvc::MsvcHeap;

/// An `MsvcHeap` over the Rust allocator, with the 16-byte alignment of the game's `operator new`.
pub struct TestHeap;

impl TestHeap {
    pub fn heap() -> MsvcHeap {
        MsvcHeap::new(Self::allocate, Self::free)
    }

    unsafe extern "C" fn allocate(size: usize) -> *mut u8 {
        // SAFETY: the layout has a non-zero size and a power-of-two alignment.
        unsafe { std::alloc::alloc(Layout::from_size_align(size, 16).unwrap()) }
    }

    unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
        // SAFETY: `MsvcHeap` passes a block from `allocate` with the size it was allocated with.
        unsafe { std::alloc::dealloc(ptr, Layout::from_size_align(size, 16).unwrap()) };
    }
}
