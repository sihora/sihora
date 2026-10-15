//! Platform shells: desktop (winit), web (wasm), mobile (Android/iOS).

mod common;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod desktop;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use desktop::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};
#[cfg(target_arch = "wasm32")]
pub use web::{run, run_app, run_reactive, run_with_text, AppFrame, TextContext};

#[cfg(target_os = "android")]
pub use mobile::{set_android_app, AndroidApp};
