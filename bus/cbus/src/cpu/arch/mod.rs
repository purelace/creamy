mod generic;

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        pub mod x86;
        pub use x86::AvailableStrategy;
    } else if #[cfg(any(target_arch = "arm", target_arch = "aarch64"))] {
        //pub mod arm;
        //pub use self::arm::AvailableStrategy;
    } else {
        pub use generic::AvailableStrategy;
    }
}
