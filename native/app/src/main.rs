#![cfg_attr(all(not(debug_assertions), windows), windows_subsystem = "windows")]
use slint::ComponentHandle;
fn main() {
    let dir = orthogrowth_core::store::pick_data_dir().expect("Aucun dossier de données inscriptible.");
    let ui = orthogrowth_app::build(dir);
    ui.run().expect("erreur d'exécution");
}
