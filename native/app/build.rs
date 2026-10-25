fn main() {
    let cfg = slint_build::CompilerConfiguration::new()
        .with_style(if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") { "material-light" } else { "fluent-light" }.into());
    slint_build::compile_with_config("ui/app.slint", cfg).unwrap();
    // Icône et informations de version de l'exécutable Windows (sans effet sous Linux).
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    embed_resource::compile("app.rc", embed_resource::NONE).manifest_optional().unwrap();
}
