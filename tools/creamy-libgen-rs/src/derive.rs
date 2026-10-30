use genco::{lang::rust::Tokens, quote};

use crate::Args;

pub fn generate_derive(args: &Args, custom: &[String]) -> Tokens {
    let mut derives = vec!["Debug".into(), "Clone".into(), "Copy".into()];

    if args.eq {
        derives.extend_from_slice(&["PartialEq".into(), "Eq".into()]);
    }

    if args.ord {
        derives.extend_from_slice(&["PartialOrd".into(), "Ord".into()]);
    }

    if args.hash {
        derives.push("Hash".into());
    }

    derives.extend_from_slice(custom);

    let list = derives.join(", ");
    quote! { #[derive($list)] }
}
