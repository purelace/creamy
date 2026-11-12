use creamy_libgen::Path;
use genco::{lang::rust::Tokens, quote};
use heck::ToSnakeCase;

use crate::{Args, path_to_string};

pub fn generate_dispatcher<P, M>(args: &Args, protocols: P, messages: M) -> Tokens
where
    P: Iterator<Item = Path>,
    M: Iterator<Item = Path>,
{
    let mut trait_functions = vec![];
    let mut match_branches = vec![];

    let mut enable_match_branches = vec![];
    let mut disable_match_branches = vec![];

    for path in protocols {
        let postfix = &match &path {
            Path::Global { name } => name.to_snake_case(),
            Path::Absolute { components } => components[components.len() - 2].to_snake_case(),
        };

        trait_functions.push(quote! {
            fn on_$(postfix)_protocol_enabled(&mut self) {}
            fn on_$(postfix)_protocol_disabled(&mut self) {}
        });

        let path = &path_to_string(&path);
        enable_match_branches.push(quote! {
            $path => {
                handler.on_$(postfix)_protocol_enabled();
            }
        });
        disable_match_branches.push(quote! {
            $path => {
                handler.on_$(postfix)_protocol_disabled();
            }
        });
    }

    for path in messages {
        let postfix = &match &path {
            Path::Global { name } => name.to_snake_case(),
            Path::Absolute { components } => components.last().unwrap().to_snake_case(),
        };
        let kind = &path_to_string(&path);
        trait_functions.push(quote! {
            fn handle_$postfix(&mut self, message: $kind);
        });

        let cast_message = args.cast_message_fn_path();
        match_branches.push(quote! {
            $kind::DISPATCH_VALUE => {
                handler.handle_$postfix($cast_message(message))
            }
        });
    }

    let untyped_message = &args.untyped_message_path();
    let delegate_trait = args.dispatch_delegate_trait_path();
    quote! {
        pub trait MessageHandler {
            $trait_functions
            fn handle_unknown_message(&mut self, dispatch_value: u32, message: $untyped_message) {}
        }

        pub struct Dispatcher<T: MessageHandler>(core::marker::PhantomData<T>);
        impl<T: MessageHandler> $delegate_trait for Dispatcher<T> {
            type Handler = T;

            #[inline(always)]
            fn dispatch_message(dispatch_value: u32, message: $untyped_message, handler: &mut Self::Handler) {
                match dispatch_value {
                    $match_branches
                    _ => handler.handle_unknown_message(dispatch_value, message)
                }
            }

            #[inline(always)]
            fn handle_on_protocol_enabled_event(protocol_id: u8, handler: &mut Self::Handler) {
                match protocol_id {
                    $enable_match_branches
                    _ => {}
                }
            }

            #[inline(always)]
            fn handle_on_protocol_disabled_event(protocol_id: u8, handler: &mut Self::Handler) {
                match protocol_id {
                    $disable_match_branches
                    _ => {}
                }
            }
        }
    }
}
