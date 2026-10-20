use crate::format::{display, Display};
use crate::table::{table, Sex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Champs du formulaire, sous forme de texte, avec les mêmes noms que `input_values` du plugin
/// (les dossiers restent donc compatibles avec la version Electron/WordPress).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Inputs {
    pub last_name: String, pub first_names: String, pub dob: String, pub exam_date: String,
    pub age: String, pub sex: String, pub bone_age: String, pub bone_method: String,
    pub age_basis: String, pub unit: String,
    pub femur_r: String, pub femur_l: String, pub tibia_r: String, pub tibia_l: String,
    pub foot_r: String, pub foot_l: String,
    pub prev_exam_date: String, pub prev_age: String,
    pub prev_femur_r: String, pub prev_femur_l: String, pub prev_tibia_r: String, pub prev_tibia_l: String,
    pub scenario: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcError { AgeRequired, Measure, Age, Previous, Date, PrevAge, Growth }
impl CalcError {
    /// Clé de traduction identique à celle du plugin (errAgeRequired, errMeasure, ...).
    pub fn key(self) -> &'static str {
        match self {
            CalcError::AgeRequired => "errAgeRequired", CalcError::Measure => "errMeasure", CalcError::Age => "errAge",
            CalcError::Previous => "errPrevious", CalcError::Date => "errDate", CalcError::PrevAge => "errPrevAge",
            CalcError::Growth => "errGrowth",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Multiplier { pub value: f64, pub interpolated: bool }

pub fn get_multiplier(age: f64, sex: Sex) -> Option<Multiplier> {
    if !age.is_finite() || age < 0.0 || age > 17.0 { return None; }
    let rows: Vec<(f64, f64)> = table().iter().filter_map(|r| r.get(sex).map(|v| (r.age, v))).collect();
    if let Some(&(_, v)) = rows.iter().find(|(a, _)| (a - age).abs() < 0.00001) {
        return Some(Multiplier { value: v, interpolated: false });
    }
    let (mut lo, mut hi) = (None, None);
    for &(a, v) in &rows {
        if a < age { lo = Some((a, v)); } else if a > age { hi = Some((a, v)); break; }
    }
    let ((la, lv), (ha, hv)) = (lo?, hi?);
    let f = (age - la) / (ha - la);
    Some(Multiplier { value: lv + (hv - lv) * f, interpolated: true })
}

#[derive(Clone, Copy, Debug)]
pub struct DevSegment {
    pub mature: f64, pub inhibition: f64, pub growth_remaining: f64, pub growth_long: f64, pub growth_short: f64,
    pub rem_r: f64, pub rem_l: f64, pub zero: bool,
}
impl DevSegment {
    fn to_json(&self) -> Value {
        if self.zero {
            json!({"mature":0,"i":0,"growth":0,"remainingRight":self.rem_r,"remainingLeft":self.rem_l})
        } else {
            json!({"mature":self.mature,"inhibition":self.inhibition,"growthRemaining":self.growth_remaining,
                   "growthLong":self.growth_long,"growthShort":self.growth_short,
                   "remainingRight":self.rem_r,"remainingLeft":self.rem_l})
        }
    }
}

fn dev_segment(cur_r: f64, cur_l: f64, old_r: f64, old_l: f64, m: f64) -> Option<DevSegment> {
    let d = cur_r - cur_l;
    let old_d = old_r - old_l;
    if d.abs() < 0.0001 {
        if old_d.abs() < 0.0001 {
            return Some(DevSegment { mature: 0.0, inhibition: 0.0, growth_remaining: 0.0, growth_long: 0.0, growth_short: 0.0,
                rem_r: cur_r * (m - 1.0), rem_l: cur_l * (m - 1.0), zero: true });
        }
        return None;
    }
    let sign = d.signum();
    let (long_now, short_now) = (cur_r.max(cur_l), cur_r.min(cur_l));
    let (long_old, short_old) = if sign > 0.0 { (old_r, old_l) } else { (old_l, old_r) };
    let (gl, gs) = (long_now - long_old, short_now - short_old);
    if gl <= 0.0 || gs < 0.0 { return None; }
    let inh = 1.0 - (gs / gl);
    let gr = long_now * (m - 1.0);
    let sr = gr * (1.0 - inh);
    Some(DevSegment { mature: sign * (d.abs() + inh * gr), inhibition: inh, growth_remaining: gr, growth_long: gl, growth_short: gs,
        rem_r: if sign > 0.0 { gr } else { sr }, rem_l: if sign > 0.0 { sr } else { gr }, zero: false })
}

/// Résultats, dans l'unité choisie par l'utilisateur (mm ou cm).
#[derive(Clone, Debug)]
pub struct Outcome {
    pub multiplier: f64, pub interpolated: bool, pub unit: String, pub scenario: String, pub sex: Sex,
    pub age_used: f64,
    pub current: f64, pub mature_total: f64, pub mature_femur: f64, pub mature_tibia: f64,
    pub growth_r: f64, pub growth_l: f64,
    pub epi_femur: Option<f64>, pub epi_tibia: Option<f64>, pub epi_both: Option<f64>,
    pub measurements: [f64; 6], // fémur D/G, tibia D/G, pied D/G (unité choisie)
    pub dev: Option<(DevSegment, DevSegment, [f64; 4])>,
}

fn num(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() { return None; }
    t.replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite())
}
fn empty(s: &str) -> bool { s.trim().is_empty() }

pub fn calculate(inp: &Inputs) -> Result<Outcome, CalcError> {
    use CalcError::*;
    let sex = Sex::parse(&inp.sex);
    let bone = inp.age_basis == "bone";
    let age_val = |s: &str| if empty(s) { f64::NAN } else { num(s).unwrap_or(f64::NAN) };
    let sel = if bone { age_val(&inp.bone_age) } else { age_val(&inp.age) };
    let mm = get_multiplier(sel, sex);
    if empty(&inp.age) { return Err(AgeRequired); }
    if bone && empty(&inp.bone_age) { return Err(AgeRequired); }
    let pos = |s: &str| num(s).filter(|v| *v > 0.0);
    let (Some(fr0), Some(fl0), Some(tr0), Some(tl0)) = (pos(&inp.femur_r), pos(&inp.femur_l), pos(&inp.tibia_r), pos(&inp.tibia_l)) else { return Err(Measure) };
    let mm = mm.ok_or(Age)?;
    let foot = |s: &str| if empty(s) { Ok(0.0) } else { num(s).ok_or(Measure) };
    let fac = if inp.unit == "cm" { 10.0 } else { 1.0 };
    let (fr, fl, trr, tl) = (fr0 * fac, fl0 * fac, tr0 * fac, tl0 * fac);
    let (foot_r, foot_l) = (foot(&inp.foot_r)? * fac, foot(&inp.foot_l)? * fac);
    let m = mm.value;
    let (d_f, d_t, d_foot) = (fr - fl, trr - tl, foot_r - foot_l);
    let d_total = d_f + d_t + d_foot;
    let (mut mature_f, mut mature_t, mature_foot) = (d_f * m, d_t * m, d_foot * m);
    let mut dev = None;
    let developmental = inp.scenario == "developmental";
    if developmental {
        let (Some(a), Some(b), Some(c), Some(d)) = (pos(&inp.prev_femur_r), pos(&inp.prev_femur_l), pos(&inp.prev_tibia_r), pos(&inp.prev_tibia_l)) else { return Err(Previous) };
        if !inp.prev_exam_date.is_empty() && !inp.exam_date.is_empty() && inp.prev_exam_date >= inp.exam_date { return Err(Date); }
        if !empty(&inp.prev_age) {
            match num(&inp.prev_age) {
                Some(pa) if pa >= 0.0 && pa < num(&inp.age).unwrap_or(f64::NAN) => {}
                _ => return Err(PrevAge),
            }
        }
        let (pfr, pfl, ptr, ptl) = (a * fac, b * fac, c * fac, d * fac);
        let f = dev_segment(fr, fl, pfr, pfl, m);
        let t = dev_segment(trr, tl, ptr, ptl, m);
        let (Some(f), Some(t)) = (f, t) else { return Err(Growth) };
        mature_f = f.mature; mature_t = t.mature;
        dev = Some((f, t, [a, b, c, d]));
    }
    let mature_total = mature_f + mature_t + mature_foot;
    let (mut growth_r, mut growth_l) = ((fr + trr) * (m - 1.0), (fl + tl) * (m - 1.0));
    if let Some((f, t, _)) = &dev { growth_r = f.rem_r + t.rem_r; growth_l = f.rem_l + t.rem_l; }

    let epi = |site: u8| -> Option<f64> {
        let k = match site { 0 => 0.71, 1 => 0.57, _ => 0.67 };
        let (long_f, long_t) = (fr.max(fl), trr.max(tl));
        let l = match site { 0 => long_f, 1 => long_t, _ => long_f + long_t };
        let disc = mature_f + mature_t;
        let discordant = mature_f * mature_t < 0.0 && mature_f.abs() > 0.01 && mature_t.abs() > 0.01;
        let denom = l * m - disc.abs() / k;
        let me = (l * m) / denom;
        if discordant || denom <= 0.0 || me < 1.0 || me > m { return None; }
        table().iter().find(|r| r.get(sex).map_or(false, |mv| mv <= me)).map(|r| r.age)
    };
    Ok(Outcome {
        multiplier: m, interpolated: mm.interpolated, unit: if inp.unit == "cm" { "cm".into() } else { "mm".into() },
        scenario: if developmental { "developmental".into() } else { "congenital".into() }, sex, age_used: sel,
        current: d_total / fac, mature_total: mature_total / fac, mature_femur: mature_f / fac, mature_tibia: mature_t / fac,
        growth_r: growth_r / fac, growth_l: growth_l / fac,
        epi_femur: epi(0), epi_tibia: epi(1), epi_both: epi(2),
        measurements: [fr / fac, fl / fac, trr / fac, tl / fac, foot_r / fac, foot_l / fac],
        dev,
    })
}

/// Construit l'enregistrement JSON (même schéma que le plugin) — `now` au format ISO 8601.
pub fn build_record(inp: &Inputs, out: &Outcome, lang: &str, now: &str) -> Value {
    let d: Display = display(out, lang);
    let n = |s: &str| num(s);
    let opt = |s: &str| if s.is_empty() { Value::Null } else { n(s).map_or(Value::Null, |v| json!(v)) };
    let dev_json = out.dev.as_ref().map(|(f, t, p)| json!({
        "femur": f.to_json(), "tibia": t.to_json(),
        "previous_measurements": {"femur_right":p[0],"femur_left":p[1],"tibia_right":p[2],"tibia_left":p[3]}
    })).unwrap_or(Value::Null);
    let me = out.measurements;
    json!({
        "application": "OrthoGrowth native", "version": env!("CARGO_PKG_VERSION"), "date": now,
        "sex": if out.sex == Sex::Girls { "girls" } else { "boys" },
        "chronological_age_years": n(&inp.age), "bone_age_years": opt(&inp.bone_age),
        "bone_age_method": if inp.bone_method.is_empty() { Value::Null } else { json!(inp.bone_method) },
        "age_basis": if inp.age_basis == "bone" { "bone" } else { "chronological" }, "age_used_years": out.age_used,
        "measurement_date": if inp.exam_date.is_empty() { Value::Null } else { json!(inp.exam_date) },
        "previous_measurement_date": if inp.prev_exam_date.is_empty() { Value::Null } else { json!(inp.prev_exam_date) },
        "previous_measurement_age_years": opt(&inp.prev_age),
        "unit": out.unit, "scenario": out.scenario,
        "measurements": {"femur_right":me[0],"femur_left":me[1],"tibia_right":me[2],"tibia_left":me[3],"foot_height_right":me[4],"foot_height_left":me[5]},
        "developmental_model": dev_json,
        "multiplier": out.multiplier, "multiplier_interpolated": out.interpolated,
        "current_discrepancy_signed_right_minus_left": out.current,
        "maturity_discrepancy_signed_right_minus_left": out.mature_total,
        "maturity_femoral_discrepancy_signed_right_minus_left": out.mature_femur,
        "maturity_tibial_discrepancy_signed_right_minus_left": out.mature_tibia,
        "estimated_remaining_growth_right": out.growth_r, "estimated_remaining_growth_left": out.growth_l,
        "epiphysiodesis_estimates_age_years": {"distal_femur": out.epi_femur, "proximal_tibia": out.epi_tibia, "combined": out.epi_both},
        "model": if out.scenario == "developmental" { "developmental multiplier model by segment; foot-height difference projected with lower-limb multiplier; epiphysiodesis timing theoretical" } else { "congenital multiplier model; epiphysiodesis timing theoretical" },
        "patient": {"last_name": inp.last_name.trim(), "first_names": inp.first_names.trim()},
        "input_values": serde_json::to_value(inp).unwrap_or(Value::Null),
        "result_display": serde_json::to_value(&d).unwrap_or(Value::Null),
    })
}
