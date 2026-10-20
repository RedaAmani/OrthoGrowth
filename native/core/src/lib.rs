//! OrthoGrowth — noyau de calcul. Port fidèle de la logique JavaScript du plugin WordPress 0.5.0.
pub mod calc;
pub mod format;
pub mod store;
pub mod table;
pub use calc::{calculate, build_record, CalcError, Inputs, Outcome};
pub use table::Sex;

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> Inputs {
        Inputs { age: "10".into(), sex: "boys".into(), unit: "mm".into(), age_basis: "chronological".into(), scenario: "congenital".into(),
            femur_r: "400".into(), femur_l: "390".into(), tibia_r: "330".into(), tibia_l: "325".into(), last_name: "T".into(), first_names: "F".into(), ..Default::default() }
    }
    #[test]
    fn exemple_du_plugin() {
        let o = calculate(&base()).unwrap();
        assert!((o.multiplier - 1.31).abs() < 1e-9);
        assert!((o.mature_total - 19.65).abs() < 1e-9);
        let d = format::display(&o, "fr");
        assert_eq!(d.current_lld, "+15 mm");
        assert_eq!(d.epi_femur_result, "14,67 ans (approx.)");
    }
    #[test]
    fn erreurs() {
        let mut i = base(); i.age = "".into();
        assert_eq!(calculate(&i).unwrap_err(), CalcError::AgeRequired);
        let mut i = base(); i.femur_l = "0".into();
        assert_eq!(calculate(&i).unwrap_err(), CalcError::Measure);
        let mut i = base(); i.age = "18".into();
        assert_eq!(calculate(&i).unwrap_err(), CalcError::Age);
    }
    #[test]
    fn arrondi_comme_intl() {
        assert_eq!(format::fmt(0.125, "en"), "0.13");
        assert_eq!(format::fmt(1.005, "en"), "1.01");
        assert_eq!(format::fmt(-5.0, "fr"), "-5");
        assert_eq!(format::fmt(1234.5, "fr"), "1\u{202F}234,5");
        assert_eq!(format::fmt(9.999, "en"), "10");
    }
    #[test]
    fn stockage_aller_retour() {
        let dir = std::env::temp_dir().join(format!("og-rs-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let s = store::Store::open(&dir).unwrap();
        let i = base();
        let rec = build_record(&i, &calculate(&i).unwrap(), "fr", &store::now_iso());
        let id = s.save(&rec).unwrap();
        assert_eq!(s.list().len(), 1);
        assert_eq!(s.get(id).unwrap()["input_values"]["femurR"], "400");
        assert_eq!(s.import_all(&s.export_all()), 1);
        s.delete(id).unwrap();
        assert_eq!(s.list().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
