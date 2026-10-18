mod generic;

cfg_select! {
    any(target_arch = "x86", target_arch = "x86_64") => {
        pub mod x86;
        pub use x86::AvailableStrategy;
    }
    any(target_arch = "arm", target_arch = "aarch64") => {
        //pub mod arm;
        //pub use self::arm::AvailableStrategy;
    }
    _ => {
        pub use generic::AvailableStrategy;
    }
}
