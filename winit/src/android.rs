//! Access the global Android application object.
//!
//! On Android, the [`AndroidApp`] handed to the entry point is the only way to
//! reach platform APIs—like the app's internal data path.
//!
//! [`AndroidApp`]: winit::platform::android::activity::AndroidApp

use std::path::PathBuf;
use std::sync::OnceLock;

use winit::platform::android::activity::AndroidApp;

pub static ANDROID_APP: OnceLock<AndroidApp> = OnceLock::new();
