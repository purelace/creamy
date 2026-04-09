mod scalar;

pub use scalar::ScalarInstructionSet;

define_strategy! {
    Fallback,
    128..  => [batch  | ScalarInstructionSet],
    1..128   => [scalar | ScalarInstructionSet],
}

pub type AvailableStrategy = Fallback;
