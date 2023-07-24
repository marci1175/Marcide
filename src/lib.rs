//! Demo-code for showing how egui is used.
//!
//! This library can be used to test 3rd party egui integrations (see for instance <https://github.com/not-fl3/egui-miniquad/blob/master/examples/demo.rs>).
//!
//! The demo is also used in benchmarks and tests.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]
#![forbid(unsafe_code)]

mod app;
pub use app::TemplateApp;

