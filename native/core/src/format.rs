use crate::calc::Outcome;
use serde::Serialize;

/// Équivalent de Intl.NumberFormat(maximumFractionDigits: 2) : arrondi « demi vers l'extérieur »
/// sur la représentation décimale la plus courte (comme ICU), virgule en français.
pub fn fmt(v: f64, lang: &str) -> String {
    if !v.is_finite() { return "NaN".into(); }
    let neg = v.is_sign_negative();
    let s = format!("{}", v.abs());
    let (int_part, frac) = match s.split_once('.') { Some((a, b)) => (a.to_string(), b.to_string()), None => (s.clone(), String::new()) };
    let mut digits: Vec<u8> = int_part.bytes().chain(frac.bytes().take(2)).map(|b| b - b'0').collect();
    let mut frac_len = frac.len().min(2);
    while digits.len() < int_part.len() + frac_len { digits.push(0); }
    if frac.len() > 2 && frac.as_bytes()[2] >= b'5' {
        let mut i = digits.len();
        loop {
            if i == 0 { digits.insert(0, 1); break; }
            i -= 1;
            if digits[i] == 9 { digits[i] = 0; } else { digits[i] += 1; break; }
        }
    }
    let int_len = digits.len() - frac_len;
    let mut frac_digits: Vec<u8> = digits.split_off(int_len);
    while frac_digits.last() == Some(&0) { frac_digits.pop(); }
    frac_len = frac_digits.len();
    let mut int_str: String = digits.iter().map(|d| (b'0' + d) as char).collect();
    // séparateur de milliers (virgule en anglais ; espace fine insécable U+202F en français)
    let sep = if lang == "fr" { '\u{202F}' } else { ',' };
    let mut grouped = String::new();
    for (i, c) in int_str.drain(..).enumerate() {
        let rem = (digits.len() - i) as usize;
        if i > 0 && rem % 3 == 0 { grouped.push(sep); }
        grouped.push(c);
    }
    let mut out = String::new();
    if neg { out.push('-'); }
    out.push_str(&grouped);
    if frac_len > 0 {
        out.push(if lang == "fr" { ',' } else { '.' });
        out.extend(frac_digits.iter().map(|d| (b'0' + d) as char));
    }
    out
}

#[derive(Clone, Debug, Serialize)]
pub struct Display {
    #[serde(rename = "multResult")] pub mult_result: String,
    #[serde(rename = "currentLLD")] pub current_lld: String,
    #[serde(rename = "matureLLD")] pub mature_lld: String,
    #[serde(rename = "matureFemur")] pub mature_femur: String,
    #[serde(rename = "matureTibia")] pub mature_tibia: String,
    pub growth: String,
    #[serde(rename = "epiFemurResult")] pub epi_femur_result: String,
    #[serde(rename = "epiTibiaResult")] pub epi_tibia_result: String,
    #[serde(rename = "epiBothResult")] pub epi_both_result: String,
}

pub fn display(o: &Outcome, lang: &str) -> Display {
    let signed = |v: f64| format!("{}{} {}", if v > 0.0 { "+" } else { "" }, fmt(v, lang), o.unit);
    let epi = |v: Option<f64>| match v {
        None => if lang == "fr" { "Non calculable avec les données disponibles".to_string() } else { "Cannot calculate from available data".to_string() },
        Some(a) => format!("{} {} (approx.)", fmt(a, lang), if lang == "fr" { "ans" } else { "years" }),
    };
    Display {
        mult_result: format!("{}{}", fmt(o.multiplier, lang), if o.interpolated { " *" } else { "" }),
        current_lld: signed(o.current), mature_lld: signed(o.mature_total), mature_femur: signed(o.mature_femur), mature_tibia: signed(o.mature_tibia),
        growth: format!("{} / {} {}", fmt(o.growth_r, lang), fmt(o.growth_l, lang), o.unit),
        epi_femur_result: epi(o.epi_femur), epi_tibia_result: epi(o.epi_tibia), epi_both_result: epi(o.epi_both),
    }
}
