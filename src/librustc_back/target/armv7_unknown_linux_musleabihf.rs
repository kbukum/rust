// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use target::{Target, TargetResult};

pub fn target() -> TargetResult {
    let mut base = super::linux_musl_base::opts();

    // Most of these settings are copied from the armv7_unknown_linux_gnueabihf
    // target.
    base.features = "+v7,+vfp3,+neon".to_string();
    base.cpu = "cortex-a8".to_string();
    base.max_atomic_width = Some(64);
    Ok(Target {
        // It's important we use "gnueabihf" and not "musleabihf" here. LLVM
        // uses it to determine the calling convention and float ABI, and LLVM
        // doesn't support the "musleabihf" value.
        llvm_target: "armv7-unknown-linux-gnueabihf".to_string(),
        target_endian: "little".to_string(),
        target_pointer_width: "32".to_string(),
        data_layout: "e-m:e-p:32:32-i64:64-v128:64:128-a:0:32-n32-S64".to_string(),
        arch: "arm".to_string(),
        target_os: "linux".to_string(),
        target_env: "musl".to_string(),
        target_vendor: "unknown".to_string(),
        options: base,
    })
}
