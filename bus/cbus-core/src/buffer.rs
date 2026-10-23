pub mod dynamic;
mod incoming;
mod outgoing;
#[doc(hidden)]
pub mod reference;
mod shared;

use alloc::alloc::alloc_zeroed;
use core::{
    alloc::Layout,
    fmt::{Debug, NumBuffer},
    marker::PhantomData,
    num::NonZeroUsize,
    ptr::NonNull,
};

use as_guard::AsGuard;
pub use incoming::{DynIncBuf, IncBuf};
pub use outgoing::{DynOutBuf, OutBuf};

use crate::{
    UntypedMessage,
    defines::{MESSAGE_SIZE, METADATA, TARGET_ALIGN},
    message::{TypedMessage, cast_message},
};

const DANGLING_LAYOUT: Layout = match Layout::from_size_align(MESSAGE_SIZE + METADATA, TARGET_ALIGN)
{
    Ok(layout) => layout,
    Err(_) => panic!("Failed to generate layout at compile-time"),
};
