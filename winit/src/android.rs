//! Access the global Android application object.
//!
//! On Android, the [`AndroidApp`] handed to the entry point is the only way to
//! reach platform APIs—like the app's internal data path.
//!
//! [`AndroidApp`]: winit::platform::android::activity::AndroidApp

use std::sync::OnceLock;

use winit::platform::android::activity::AndroidApp;

/// The application object `android_main` received.
///
/// Set by [`run_android`](crate::run_android) before the event loop starts, so
/// code loaded later can reach platform APIs such as the app's data path.
pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();
