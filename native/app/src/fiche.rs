//! Fiche imprimable : une page HTML autonome, à ouvrir dans le navigateur (Imprimer / Enregistrer en PDF).
use serde_json::Value;
use std::collections::HashMap;

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn html(rec: &Value, t: &HashMap<String, String>) -> String {
    let g = |k: &str| t.get(k).cloned().unwrap_or_default();
    let iv = |k: &str| rec["input_values"][k].as_str().unwrap_or("").to_string();
    let unit = rec["unit"].as_str().unwrap_or("mm").to_string();
    let patient = format!("{} {}", rec["patient"]["last_name"].as_str().unwrap_or(""), rec["patient"]["first_names"].as_str().unwrap_or(""));
    let sex = if iv("sex") == "girls" { g("girls") } else { g("boys") };
    let basis = if iv("ageBasis") == "bone" { g("bone") } else { g("chronological") };
    let scenario = if iv("scenario") == "developmental" { g("developmental") } else { g("congenital") };
    let mut rows = String::new();
    let mut add = |k: &str, v: String| { if !v.trim().is_empty() { rows += &format!("<tr><th>{}</th><td>{}</td></tr>", esc(k), esc(&v)); } };
    add(&g("dobLabel"), iv("dob")); add(&g("examLabel"), iv("examDate")); add(&g("ageLabel"), iv("age"));
    add(&g("sexLabel"), sex); add(&g("boneAgeLabel"), iv("boneAge")); add(&g("boneMethodLabel"), iv("boneMethod"));
    add(&g("ageBasisLabel"), basis); add(&g("unitLabel"), unit.clone()); add(&g("scenarioLabel"), scenario);
    let seg = |label: &str, r: &str, l: &str| format!("<tr><th>{}</th><td>{}</td><td>{}</td></tr>", esc(label), esc(&iv(r)), esc(&iv(l)));
    let mut meas = format!("<tr><th>{}</th><th>{}</th><th>{}</th></tr>", esc(&g("segmentHead")), esc(&g("rightHead")), esc(&g("leftHead")));
    meas += &seg(&g("femurLabel"), "femurR", "femurL");
    meas += &seg(&g("tibiaLabel"), "tibiaR", "tibiaL");
    if !iv("footR").is_empty() || !iv("footL").is_empty() { meas += &seg(&g("footLabel"), "footR", "footL"); }
    if rec["scenario"] == "developmental" {
        meas += &format!("<tr><th colspan=3>{} — {} {}</th></tr>", esc(&g("previousTitle")), esc(&iv("prevExamDate")), esc(&iv("prevAge")));
        meas += &seg(&g("previousFemurLabel"), "prevFemurR", "prevFemurL");
        meas += &seg(&g("previousTibiaLabel"), "prevTibiaR", "prevTibiaL");
    }
    let rd = |k: &str| rec["result_display"][k].as_str().unwrap_or("—").to_string();
    let res: Vec<(&str, &str)> = vec![("multResultLabel", "multResult"), ("currentLLDLabel", "currentLLD"), ("matureLLDLabel", "matureLLD"),
        ("matureFemurLabel", "matureFemur"), ("matureTibiaLabel", "matureTibia"), ("growthLabel", "growth"),
        ("epiFemurLabel", "epiFemurResult"), ("epiTibiaLabel", "epiTibiaResult"), ("epiBothLabel", "epiBothResult")];
    let results: String = res.iter().map(|(l, k)| format!("<tr><th>{}</th><td><b>{}</b></td></tr>", esc(&g(l)), esc(&rd(k)))).collect();
    format!(r#"<!doctype html><html><head><meta charset="utf-8"><title>{title} — {patient}</title>
<style>body{{font-family:system-ui,Segoe UI,Arial,sans-serif;max-width:820px;margin:24px auto;padding:0 16px;color:#17202b}}
h1{{font-size:22px;margin:0 0 4px}}h2{{font-size:16px;margin:22px 0 8px;border-bottom:1px solid #ccd5df;padding-bottom:4px}}
table{{border-collapse:collapse;width:100%;font-size:14px}}th,td{{border-bottom:1px solid #e1e7ed;padding:6px 8px;text-align:left;vertical-align:top}}th{{width:46%;font-weight:600;background:#f4f7fa}}
.note{{border-left:4px solid #b7791f;background:#fff7e6;padding:10px 12px;margin:16px 0;font-size:13px}}.muted{{color:#526273;font-size:12px}}
@media print{{body{{margin:0}}}}</style></head><body>
<h1>{title}</h1><div class="muted">{dword} : {date}</div>
<h2>{pword} : {patient}</h2><table>{rows}</table>
<h2>{inputs}</h2><table>{meas}</table>
<h2>{rtitle}</h2><table>{results}</table>
<div class="note">{note}</div><div class="muted">{method}</div></body></html>"#,
        title = esc(&g("ficheTitle")), patient = esc(patient.trim()), dword = esc(&g("dateWord")), date = esc(rec["date"].as_str().unwrap_or("")),
        pword = esc(&g("patientWord")), rows = rows, inputs = esc(&g("inputsWord")), meas = meas, rtitle = esc(&g("resultsTitle")),
        results = results, note = esc(&g("medicalNote")), method = esc(&g("methodText")))
}
