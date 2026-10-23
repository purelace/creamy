#! /bin/bash

CARGO_PROFILE_RELEASE_LTO=false cargo +nightly fuzz run send_messages
