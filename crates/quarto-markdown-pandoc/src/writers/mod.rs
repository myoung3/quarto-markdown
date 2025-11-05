/*
 * mod.rs
 * Copyright (c) 2025 Posit, PBC
 */

#[cfg(feature = "terminal-support")]
pub mod ansi;
pub mod html;
pub mod json;
pub mod json_block;
pub mod native;
pub mod qmd;
pub mod r;
