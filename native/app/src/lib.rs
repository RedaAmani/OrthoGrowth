slint::include_modules!();
#[cfg(feature = "android-glue")]
mod android_glue;
mod dates;
mod fiche;
mod tr_gen;

use orthogrowth_core::format::{display, fmt, Display};
use orthogrowth_core::store::{now_iso, Store};
use orthogrowth_core::table::table;
use orthogrowth_core::{build_record, calculate, Inputs};
use serde_json::{json, Value};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

type Strings = HashMap<String, HashMap<String, String>>;

const SEX: [&str; 2] = ["boys", "girls"];
const BASIS: [&str; 2] = ["chronological", "bone"];
const UNIT: [&str; 2] = ["mm", "cm"];
const METHOD: [&str; 5] = ["", "Greulich-Pyle", "Tanner-Whitehouse", "Dimeglio", "Other"];
const SCEN: [&str; 2] = ["congenital", "developmental"];
const URLS: [&str; 6] = [
    "https://link.springer.com/article/10.1007/s11832-016-0754-4",
    "https://link.springer.com/article/10.1007/s11832-016-0754-4/tables/1",
    "https://pubmed.ncbi.nlm.nih.gov/11057472/",
    "https://github.com/RedaAmani/OrthoGrowth",
    "https://buymeacoffee.com/dramani",
    "https://creativecommons.org/licenses/by/4.0/",
];

#[derive(Clone)]
struct Last { inputs: Inputs, record: Value }

struct State { store: Store, lang: String, strings: Strings, last: Option<Last>, config: PathBuf }
impl State {
    fn t(&self, k: &str) -> String { self.strings[&self.lang].get(k).cloned().unwrap_or_else(|| k.to_string()) }
}

fn idx(list: &[&str], s: &str) -> i32 { list.iter().position(|x| *x == s).unwrap_or(0) as i32 }
fn pick(list: &[&str], i: i32) -> String { list.get(i.max(0) as usize).unwrap_or(&list[0]).to_string() }
fn model(items: Vec<String>) -> ModelRc<SharedString> { ModelRc::new(VecModel::from(items.into_iter().map(SharedString::from).collect::<Vec<_>>())) }

fn read_inputs(ui: &MainWindow) -> Inputs {
    let f = ui.get_form();
    Inputs {
        last_name: f.last_name.to_string(), first_names: f.first_names.to_string(), dob: f.dob.to_string(), exam_date: f.exam_date.to_string(),
        age: f.age.to_string(), sex: pick(&SEX, f.sex), bone_age: f.bone_age.to_string(), bone_method: pick(&METHOD, f.bone_method),
        age_basis: pick(&BASIS, f.age_basis), unit: pick(&UNIT, f.unit),
        femur_r: f.femur_r.to_string(), femur_l: f.femur_l.to_string(), tibia_r: f.tibia_r.to_string(), tibia_l: f.tibia_l.to_string(),
        foot_r: f.foot_r.to_string(), foot_l: f.foot_l.to_string(),
        prev_exam_date: f.prev_exam_date.to_string(), prev_age: f.prev_age.to_string(),
        prev_femur_r: f.prev_femur_r.to_string(), prev_femur_l: f.prev_femur_l.to_string(),
        prev_tibia_r: f.prev_tibia_r.to_string(), prev_tibia_l: f.prev_tibia_l.to_string(),
        scenario: pick(&SCEN, f.scenario),
    }
}

fn write_inputs(ui: &MainWindow, i: &Inputs) {
    let s = |x: &String| SharedString::from(x.as_str());
    ui.set_form(Form {
        last_name: s(&i.last_name), first_names: s(&i.first_names), dob: s(&i.dob), exam_date: s(&i.exam_date), age: s(&i.age), bone_age: s(&i.bone_age),
        femur_r: s(&i.femur_r), femur_l: s(&i.femur_l), tibia_r: s(&i.tibia_r), tibia_l: s(&i.tibia_l), foot_r: s(&i.foot_r), foot_l: s(&i.foot_l),
        prev_exam_date: s(&i.prev_exam_date), prev_age: s(&i.prev_age),
        prev_femur_r: s(&i.prev_femur_r), prev_femur_l: s(&i.prev_femur_l), prev_tibia_r: s(&i.prev_tibia_r), prev_tibia_l: s(&i.prev_tibia_l),
        sex: idx(&SEX, &i.sex), age_basis: idx(&BASIS, &i.age_basis), unit: idx(&UNIT, &i.unit), bone_method: idx(&METHOD, &i.bone_method),
        scenario: idx(&SCEN, &i.scenario),
    });
}

fn results_from(d: &Display) -> Results {
    Results { mult: d.mult_result.as_str().into(), current: d.current_lld.as_str().into(), mature: d.mature_lld.as_str().into(),
        mature_femur: d.mature_femur.as_str().into(), mature_tibia: d.mature_tibia.as_str().into(), growth: d.growth.as_str().into(),
        epi_femur: d.epi_femur_result.as_str().into(), epi_tibia: d.epi_tibia_result.as_str().into(), epi_both: d.epi_both_result.as_str().into() }
}
fn results_from_value(v: &Value) -> Option<Results> {
    let g = |k: &str| v[k].as_str().map(SharedString::from);
    Some(Results { mult: g("multResult")?, current: g("currentLLD")?, mature: g("matureLLD")?, mature_femur: g("matureFemur")?,
        mature_tibia: g("matureTibia")?, growth: g("growth")?, epi_femur: g("epiFemurResult")?, epi_tibia: g("epiTibiaResult")?, epi_both: g("epiBothResult")? })
}
fn clear_results(ui: &MainWindow) {
    let d: SharedString = "—".into();
    ui.set_results(Results { mult: d.clone(), current: d.clone(), mature: d.clone(), mature_femur: d.clone(), mature_tibia: d.clone(), growth: d.clone(), epi_femur: d.clone(), epi_tibia: d.clone(), epi_both: d });
}
fn set_status(ui: &MainWindow, msg: &str, err: bool) { ui.set_status(msg.into()); ui.set_status_error(err); }
fn set_rec_status(ui: &MainWindow, msg: &str, err: bool) { ui.set_record_status(msg.into()); ui.set_record_status_error(err); }

fn fill_table(ui: &MainWindow, lang: &str) {
    let rows: Vec<TableRow> = table().iter().filter(|r| r.boys.is_some() || r.girls.is_some()).map(|r| {
        let f = |v: Option<f64>| v.map(|x| fmt(x, lang)).unwrap_or_else(|| "—".into());
        TableRow { age: fmt(r.age, lang).into(), boys: f(r.boys).into(), girls: f(r.girls).into() }
    }).collect();
    ui.set_table_rows(ModelRc::new(VecModel::from(rows)));
}

fn refresh_records(ui: &MainWindow, st: &State) {
    let items: Vec<RecordItem> = st.store.list().into_iter().map(|r| {
        let name = format!("{} {}", r.last_name, r.first_names);
        let name = if name.trim().is_empty() { st.t("unnamed") } else { name.trim().to_string() };
        let date = r.measurement_date.map(|d| format!(" · {d}")).unwrap_or_default();
        let sc = if r.scenario == "developmental" { st.t("devLabel") } else { st.t("congLabel") };
        RecordItem { id: r.id as i32, label: format!("{name}{date} · {sc}").into() }
    }).collect();
    if !items.iter().any(|i| i.id == ui.get_selected_id()) { ui.set_selected_id(-1); }
    ui.set_records(ModelRc::new(VecModel::from(items)));
}

fn apply_lang(ui: &MainWindow, st: &State) {
    let t = &st.strings[&st.lang];
    tr_gen::apply(ui, t);
    let g = ui.global::<T>();
    g.set_sex_options(model(vec![t["boys"].clone(), t["girls"].clone()]));
    g.set_basis_options(model(vec![t["chronological"].clone(), t["bone"].clone()]));
    g.set_scenario_options(model(vec![t["congenital"].clone(), t["developmental"].clone()]));
    g.set_method_options(model(vec![t["notSpecified"].clone(), "Greulich-Pyle".into(), "Tanner-Whitehouse".into(), "Dimeglio".into(), t["methodOther"].clone()]));
    fill_table(ui, &st.lang);
    refresh_records(ui, st);
}

fn do_calculate(ui: &MainWindow, st: &mut State) {
    let inp = read_inputs(ui);
    match calculate(&inp) {
        Ok(out) => {
            set_results(ui, &display(&out, &st.lang));
            let rec = build_record(&inp, &out, &st.lang, &now_iso());
            let mut msg = st.t("done");
            if out.interpolated { msg.push_str(" *"); }
            if out.age_used >= 10.0 { msg.push_str(&st.t("ageNote")); }
            st.last = Some(Last { inputs: inp, record: rec });
            set_status(ui, &msg, false);
        }
        Err(e) => { clear_results(ui); st.last = None; set_status(ui, &st.t(e.key()), true); }
    }
}
fn set_results(ui: &MainWindow, d: &Display) { ui.set_results(results_from(d)); }

/// Après un changement de langue : réaffiche les résultats courants et les messages dans la nouvelle langue.
fn relocalize(ui: &MainWindow, st: &mut State) {
    set_status(ui, "", false);
    set_rec_status(ui, "", false);
    let Some(mut last) = st.last.clone() else { return };
    if let Ok(out) = calculate(&last.inputs) {
        let d = display(&out, &st.lang);
        ui.set_results(results_from(&d));
        last.record["result_display"] = serde_json::to_value(&d).unwrap_or(Value::Null);
        let mut msg = st.t("done");
        if out.interpolated { msg.push_str(" *"); }
        if out.age_used >= 10.0 { msg.push_str(&st.t("ageNote")); }
        set_status(ui, &msg, false);
        st.last = Some(last);
    }
}

fn do_save(ui: &MainWindow, st: &mut State) {
    let Some(last) = st.last.clone() else { return set_rec_status(ui, &st.t("needCalcSave"), true) };
    let inp = read_inputs(ui);
    let (ln, fnm) = (inp.last_name.trim().to_string(), inp.first_names.trim().to_string());
    if ln.is_empty() || fnm.is_empty() { return set_rec_status(ui, &st.t("needNames"), true); }
    if serde_json::to_value(&last.inputs).ok() != serde_json::to_value(&inp).ok() { return set_rec_status(ui, &st.t("staleCalc"), true); }
    let mut rec = last.record.clone();
    rec["patient"] = json!({"last_name": ln, "first_names": fnm});
    rec["input_values"] = serde_json::to_value(&inp).unwrap_or(Value::Null);
    rec["saved_at"] = json!(now_iso());
    match st.store.save(&rec) {
        Ok(id) => {
            rec["id"] = json!(id);
            st.last = Some(Last { inputs: inp, record: rec });
            ui.set_selected_id(id as i32);
            refresh_records(ui, st);
            set_rec_status(ui, &st.t("savedOk"), false);
        }
        Err(e) => set_rec_status(ui, &format!("{} ({e})", st.t("storageError")), true),
    }
}

fn do_load(ui: &MainWindow, st: &mut State) {
    let id = ui.get_selected_id();
    if id < 0 { return set_rec_status(ui, &st.t("selectRecord"), true); }
    match st.store.get(id as u64) {
        Ok(rec) => {
            if rec["input_values"].is_null() { return set_rec_status(ui, &st.t("storageError"), true); }
            let inp: Inputs = serde_json::from_value(rec["input_values"].clone()).unwrap_or_default();
            write_inputs(ui, &inp);
            match results_from_value(&rec["result_display"]) { Some(r) => ui.set_results(r), None => clear_results(ui) }
            st.last = Some(Last { inputs: inp, record: rec });
            let m = st.t("loadedOk");
            set_status(ui, &m, false);
            set_rec_status(ui, &m, false);
            ui.set_tab(0);
        }
        Err(e) => set_rec_status(ui, &e, true),
    }
}

fn do_delete(ui: &MainWindow, st: &mut State) {
    ui.set_confirm_delete(false);
    let id = ui.get_selected_id();
    if id < 0 { return set_rec_status(ui, &st.t("selectRecord"), true); }
    match st.store.delete(id as u64) {
        Ok(()) => { ui.set_selected_id(-1); refresh_records(ui, st); set_rec_status(ui, &st.t("deletedOk"), false); }
        Err(e) => set_rec_status(ui, &e, true),
    }
}

fn safe_name(rec: &Value) -> String {
    let raw = format!("{}-{}", rec["patient"]["last_name"].as_str().unwrap_or(""), rec["patient"]["first_names"].as_str().unwrap_or(""));
    let mut out = String::new();
    let mut last_us = false;
    for c in raw.trim().chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' { out.push(c); last_us = false; } else if !last_us { out.push('_'); last_us = true; }
    }
    if out.trim_matches(|c| c == '_' || c == '-').is_empty() { "record".into() } else { out }
}
fn stamp() -> String { now_iso().chars().filter(|c| c.is_ascii_digit()).take(14).collect() }

fn write_file(dir: &Path, sub: &str, name: &str, content: &str) -> std::io::Result<PathBuf> {
    let d = dir.join(sub);
    std::fs::create_dir_all(&d)?;
    let p = d.join(name);
    std::fs::write(&p, content)?;
    Ok(p)
}

fn do_export(ui: &MainWindow, st: &mut State) {
    let Some(last) = &st.last else { return set_status(ui, &st.t("needCalcFirst"), true) };
    let name = format!("OrthoGrowth-{}-{}.json", safe_name(&last.record), stamp());
    let body = serde_json::to_string_pretty(&last.record).unwrap_or_default();
    match write_file(&st.store.dir, "exports", &name, &body) {
        Ok(p) => set_rec_status(ui, &format!("{} {}", st.t("exportedTo"), p.display()), false),
        Err(e) => set_rec_status(ui, &format!("{} ({e})", st.t("storageError")), true),
    }
}

fn do_fiche(ui: &MainWindow, st: &mut State) {
    let Some(last) = &st.last else { return set_status(ui, &st.t("needCalcFirst"), true) };
    let html = fiche::html(&last.record, &st.strings[&st.lang]);
    let name = format!("fiche-{}-{}.html", safe_name(&last.record), stamp());
    match write_file(&st.store.dir, "fiches", &name, &html) {
        Ok(p) => { open_path(&p); set_rec_status(ui, &format!("{} {}", st.t("ficheOpened"), p.display()), false); }
        Err(e) => set_rec_status(ui, &format!("{} ({e})", st.t("storageError")), true),
    }
}

fn open_path(p: &Path) {
    #[cfg(windows)] { let _ = std::process::Command::new("explorer").arg(p).spawn(); }
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(p).spawn(); }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))] { let _ = std::process::Command::new("xdg-open").arg(p).spawn(); }
    #[cfg(target_os = "android")] { let _ = p; } // pas de gestionnaire de fichiers à ouvrir depuis l'application
}
fn open_url(u: &str) {
    #[cfg(windows)] { let _ = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", u]).spawn(); }
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(u).spawn(); }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))] { let _ = std::process::Command::new("xdg-open").arg(u).spawn(); }
    #[cfg(all(target_os = "android", feature = "android-glue"))] { let _ = android_glue::open_url(u); }
    #[cfg(not(any(unix, windows)))] { let _ = u; }
}

fn do_dates(ui: &MainWindow) {
    let mut f = ui.get_form();
    let mut changed = false;
    if let Some(a) = dates::decimal_age(&f.dob, &f.exam_date) { let s = format!("{a:.2}"); if f.age != s.as_str() { f.age = s.into(); changed = true; } }
    if let Some(a) = dates::decimal_age(&f.dob, &f.prev_exam_date) { let s = format!("{a:.2}"); if f.prev_age != s.as_str() { f.prev_age = s.into(); changed = true; } }
    if changed { ui.set_form(f); }
}

fn do_reset(ui: &MainWindow, st: &mut State) {
    write_inputs(ui, &Inputs::default());
    clear_results(ui);
    st.last = None;
    set_status(ui, "", false);
    set_rec_status(ui, "", false);
}

fn detect_lang(config: &Path) -> String {
    if let Ok(s) = std::fs::read_to_string(config) { let s = s.trim(); if s == "fr" || s == "en" { return s.to_string(); } }
    #[cfg(all(target_os = "android", feature = "android-glue"))]
    if let Some(sys) = android_glue::system_language() { return if sys == "fr" { "fr".into() } else { "en".into() }; }
    let env = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"].iter().filter_map(|k| std::env::var(k).ok()).find(|v| !v.is_empty());
    match env { Some(v) => if v.to_lowercase().starts_with("fr") { "fr".into() } else { "en".into() }, None => "fr".into() }
}

pub fn build(data_dir: PathBuf) -> MainWindow {
    let ui = MainWindow::new().expect("fenêtre");
    let store = Store::open(&data_dir).expect("dossier de données");
    let config = data_dir.parent().map(|p| p.join("orthogrowth.cfg")).unwrap_or_else(|| data_dir.join("orthogrowth.cfg"));
    let strings: Strings = serde_json::from_str(include_str!("../strings.json")).expect("textes");
    let lang = detect_lang(&config);
    let st = Rc::new(RefCell::new(State { store, lang: lang.clone(), strings, last: None, config }));
    ui.set_lang_index(if lang == "en" { 1 } else { 0 });
    ui.set_data_path(data_dir.display().to_string().into());
    ui.set_app_version(env!("CARGO_PKG_VERSION").into());
    {
        // Icône (128×128 RGBA brut, intégré au programme) : fenêtre et panneau « À propos ».
        let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(128, 128);
        for (dst, src) in buf.make_mut_slice().iter_mut().zip(include_bytes!("../assets/icon-128.rgba").chunks_exact(4)) {
            *dst = slint::Rgba8Pixel::new(src[0], src[1], src[2], src[3]);
        }
        ui.set_app_icon(slint::Image::from_rgba8(buf));
    }
    ui.on_quit(|| { let _ = slint::quit_event_loop(); });
    clear_results(&ui);
    apply_lang(&ui, &st.borrow());

    macro_rules! wire { ($setter:ident, $f:expr) => {{ let w = ui.as_weak(); let s = st.clone(); ui.$setter(move || { if let Some(u) = w.upgrade() { $f(&u, &mut s.borrow_mut()); } }); }}; }
    wire!(on_calculate, do_calculate);
    wire!(on_save_record, do_save);
    wire!(on_load_record, do_load);
    wire!(on_delete_record, do_delete);
    wire!(on_reset, do_reset);
    wire!(on_export_json, do_export);
    wire!(on_open_fiche, do_fiche);
    wire!(on_refresh_records, |u: &MainWindow, s: &mut State| refresh_records(u, s));
    wire!(on_open_folder, |_u: &MainWindow, s: &mut State| open_path(&s.store.dir));
    wire!(on_delete_request, |u: &MainWindow, s: &mut State| {
        if u.get_selected_id() < 0 { set_rec_status(u, &s.t("selectRecord"), true); } else { u.set_confirm_delete(true); }
    });
    { let w = ui.as_weak(); ui.on_dates_changed(move || { if let Some(u) = w.upgrade() { do_dates(&u); } }); }
    { let w = ui.as_weak(); ui.on_open_url(move |i| { let _ = &w; if let Some(u) = URLS.get(i.max(0) as usize) { open_url(u); } }); }
    {
        let (w, s) = (ui.as_weak(), st.clone());
        ui.on_lang_changed(move |i| {
            if let Some(u) = w.upgrade() {
                let mut s = s.borrow_mut();
                s.lang = if i == 1 { "en".into() } else { "fr".into() };
                let _ = std::fs::write(&s.config, &s.lang);
                apply_lang(&u, &s);
                relocalize(&u, &mut s);
            }
        });
    }
    ui
}

/// Point d'entrée Android, appelé par le crate `orthogrowth-android`.
#[cfg(target_os = "android")]
pub fn android_start(app: slint::android::AndroidApp) {
    #[cfg(feature = "android-glue")]
    android_glue::register(app.vm_as_ptr(), app.activity_as_ptr());
    slint::android::init(app.clone()).expect("initialisation Android");
    // Dossier propre à l'application : visible en USB sous Android/data/<paquet>/files, sinon espace interne.
    let base = app.external_data_path().or_else(|| app.internal_data_path()).expect("aucun dossier de données");
    let ui = build(base.join("donnees"));
    ui.run().expect("erreur d'exécution");
}
