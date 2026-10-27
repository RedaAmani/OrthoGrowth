//! Pont JNI minimal vers Android : ouvrir un lien et lire la langue du système.
//! Compilé seulement avec la fonctionnalité `android-glue` (activée par le crate `android`).
#![allow(dead_code)]
use jni::errors::Result as JniResult;
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::ffi::c_void;
use std::sync::OnceLock;

static HANDLES: OnceLock<(usize, usize)> = OnceLock::new();

/// Mémorise la machine virtuelle Java et l'activité (pointeurs fournis par android-activity).
pub fn register(vm: *mut c_void, activity: *mut c_void) {
    let _ = HANDLES.set((vm as usize, activity as usize));
}

fn with_activity<T, F>(f: F) -> Option<T>
where
    F: for<'a, 'b, 'c> FnOnce(&'a mut JNIEnv<'b>, &JObject<'c>) -> JniResult<T>,
{
    let &(vm, act) = HANDLES.get()?;
    let vm = unsafe { JavaVM::from_raw(vm as *mut jni::sys::JavaVM) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let activity = unsafe { JObject::from_raw(act as jni::sys::jobject) };
    match f(&mut env, &activity) {
        Ok(v) => Some(v),
        Err(_) => {
            let _ = env.exception_clear();
            None
        }
    }
}

/// Ouvre une adresse web dans le navigateur (intent ACTION_VIEW).
pub fn open_url(url: &str) -> bool {
    with_activity(|env, activity| {
        let juri = env.new_string(url)?;
        let uri = env
            .call_static_method("android/net/Uri", "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::Object(&juri)])?
            .l()?;
        let action = env.new_string("android.intent.action.VIEW")?;
        let intent = env.new_object(
            "android/content/Intent",
            "(Ljava/lang/String;Landroid/net/Uri;)V",
            &[JValue::Object(&action), JValue::Object(&uri)],
        )?;
        env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&intent)])?;
        Ok(())
    })
    .is_some()
}

/// Langue du système (« fr », « en »…), si disponible.
pub fn system_language() -> Option<String> {
    with_activity(|env, _| {
        let locale = env.call_static_method("java/util/Locale", "getDefault", "()Ljava/util/Locale;", &[])?.l()?;
        let lang = env.call_method(&locale, "getLanguage", "()Ljava/lang/String;", &[])?.l()?;
        let s: String = env.get_string(&JString::from(lang))?.into();
        Ok(s)
    })
}

/// Ouvre le menu de partage d'Android avec un texte (intent ACTION_SEND).
pub fn share_text(title: &str, text: &str) -> bool {
    with_activity(|env, activity| {
        let action = env.new_string("android.intent.action.SEND")?;
        let send = env.new_object("android/content/Intent", "(Ljava/lang/String;)V", &[JValue::Object(&action)])?;
        let mime = env.new_string("text/plain")?;
        env.call_method(&send, "setType", "(Ljava/lang/String;)Landroid/content/Intent;", &[JValue::Object(&mime)])?;
        for (key, value) in [("android.intent.extra.TEXT", text), ("android.intent.extra.SUBJECT", title)] {
            let k = env.new_string(key)?;
            let v = env.new_string(value)?;
            env.call_method(
                &send,
                "putExtra",
                "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                &[JValue::Object(&k), JValue::Object(&v)],
            )?;
        }
        let jt = env.new_string(title)?;
        let chooser = env
            .call_static_method(
                "android/content/Intent",
                "createChooser",
                "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
                &[JValue::Object(&send), JValue::Object(&jt)],
            )?
            .l()?;
        env.call_method(activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&chooser)])?;
        Ok(())
    })
    .is_some()
}
