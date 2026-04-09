mod buffer_pool;
mod layout;
mod msg_pool;

pub use buffer_pool::BufferPool;
pub use msg_pool::MessagePool;

#[macro_export]
macro_rules! align_debug_assert {
    ($ptr:expr, $align:expr) => {
        debug_assert!(
            ($ptr as usize).is_multiple_of($align),
            "Address '{:#?} ({})' is not aligned",
            $ptr,
            $ptr as usize
        );
    };
}
