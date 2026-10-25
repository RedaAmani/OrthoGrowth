//! Point d'entrée Android : `android_main` est appelée par `android-activity` au démarrage.
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    orthogrowth_app::android_start(app);
}
