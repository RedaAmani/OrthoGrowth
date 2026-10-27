use std::path::PathBuf;

fn main() {
    let android = std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android");
    println!("cargo:rerun-if-env-changed=ORTHOGROWTH_STYLE");
    println!("cargo:rerun-if-env-changed=ORTHOGROWTH_STRIP_DESKTOP");
    // ORTHOGROWTH_STRIP_DESKTOP : essai sur ordinateur de la version sans menus.
    let strip = android || std::env::var("ORTHOGROWTH_STRIP_DESKTOP").is_ok();
    println!("cargo:rerun-if-changed=ui/app.slint");
    // Style : Material (grandes zones tactiles) sur Android, Fluent ailleurs ; ORTHOGROWTH_STYLE force un style (essais).
    let style = std::env::var("ORTHOGROWTH_STYLE")
        .unwrap_or_else(|_| (if android { "material-light" } else { "fluent-light" }).to_string());
    let mut cfg = slint_build::CompilerConfiguration::new().with_style(style);
    let mut source = PathBuf::from("ui/app.slint");
    if strip {
        // Pas de barre de menus sur téléphone : on retire le bloc balisé DESKTOP-ONLY.
        let text = std::fs::read_to_string("ui/app.slint").expect("lecture de app.slint");
        let mut out = String::new();
        let mut skip = false;
        for line in text.lines() {
            if line.contains("DESKTOP-ONLY-BEGIN") { skip = true; continue; }
            if line.contains("DESKTOP-ONLY-END") { skip = false; continue; }
            if !skip { out.push_str(line); out.push('\n'); }
        }
        let dst = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("app_android.slint");
        std::fs::write(&dst, out).expect("écriture de app_android.slint");
        cfg = cfg.with_include_paths(vec![PathBuf::from("ui")]);
        source = dst;
    }
    slint_build::compile_with_config(source, cfg).unwrap();
    // Icône et informations de version de l'exécutable Windows (sans effet ailleurs).
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    embed_resource::compile("app.rc", embed_resource::NONE).manifest_optional().unwrap();
}
