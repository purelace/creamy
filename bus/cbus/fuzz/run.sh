#! /bin/bash

CARGO_PROFILE_RELEASE_LTO=false cargo +nightly fuzz run fuzz_target_1
