# Compiler l'exécutable Windows depuis Ubuntu 24.04 (méthode utilisée ici)

Utile seulement si vous n'utilisez pas GitHub Actions ni un PC Windows. Paquets : rustc-1.89, cargo-1.89,
rust-1.89-src, gcc-mingw-w64-x86-64-win32, binutils-mingw-w64-x86-64. Slint est figé à 1.16.1 (Cargo.lock) car 1.17+ exige Rust 1.92.

1. Les sources Rust d'Ubuntu omettent la dépendance de `std` vers `windows_targets`. Dans
   `/usr/lib/rust-1.89/lib/rustlib/src/rust/library/std/Cargo.toml`, ajouter avant `[dev-dependencies]` :
       [target.'cfg(windows)'.dependencies]
       windows-targets = { path = "../windows_targets" }
2. Fournir les objets de démarrage manquants :
       D=/usr/lib/rust-1.89/lib/rustlib/x86_64-pc-windows-gnu/lib; mkdir -p $D
       for f in rsbegin rsend; do RUSTC_BOOTSTRAP=1 rustc --edition 2021 --target x86_64-pc-windows-gnu \
         --emit=obj -C opt-level=2 -C panic=abort -o $D/$f.o /usr/lib/rust-1.89/lib/rustlib/src/rust/library/rtstartup/$f.rs; done
3. Compiler :
       RUSTC_BOOTSTRAP=1 cargo build --release -p orthogrowth --target x86_64-pc-windows-gnu -Zbuild-std=std,panic_abort
