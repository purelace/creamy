use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::get_crate_root;

pub const MESSAGE_BUS_SUBSCRIBER_CRATE: &str = "message-bus-subscriber";

#[must_use]
#[inline(always)]
pub fn MetadataDerive() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::macros::Metadata)
}

#[must_use]
#[inline(always)]
pub fn Destination() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::Destination)
}

#[must_use]
#[inline(always)]
pub fn MessageType() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::message::MessageType)
}

#[must_use]
#[inline(always)]
pub fn Bytemuck() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::bytemuck)
}

#[must_use]
#[inline(always)]
pub fn TypedMessage() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::message::TypedMessage)
}

#[must_use]
#[inline(always)]
pub fn Metadata() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::message::Metadata)
}

#[must_use]
#[inline(always)]
pub fn MessageSize() -> TokenStream2 {
    let crate_name = get_crate_root(MESSAGE_BUS_SUBSCRIBER_CRATE);
    quote!(#crate_name::MESSAGE_SIZE)
}
