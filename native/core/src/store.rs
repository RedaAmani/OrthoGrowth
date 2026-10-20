//! Stockage en fichiers : un `dossier-<id>.json` lisible par enregistrement, dans un dossier `donnees`.
//! Format identique à la version Electron : les dossiers sont échangeables entre les deux.
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store { pub dir: PathBuf }

#[derive(Debug, Clone)]
pub struct Summary { pub id: u64, pub last_name: String, pub first_names: String, pub measurement_date: Option<String>, pub scenario: String, pub updated_at: String }

/// Horodatage ISO 8601 UTC, sans dépendance externe.
pub fn now_iso() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, secs) = (d.div_euclid(86400), d.rem_euclid(86400));
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, secs / 3600, (secs % 3600) / 60, secs % 60)
}

/// À côté de l'exécutable (`donnees/`) ; sinon Documents/OrthoGrowth/donnees si ce dossier n'est pas inscriptible.
pub fn pick_data_dir() -> io::Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() { if let Some(p) = exe.parent() { candidates.push(p.join("donnees")); } }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from);
    if let Some(h) = home { candidates.push(h.join("Documents").join("OrthoGrowth").join("donnees")); }
    for c in candidates {
        if fs::create_dir_all(&c).is_ok() {
            let probe = c.join(".ecriture-test");
            if fs::write(&probe, b"x").is_ok() { let _ = fs::remove_file(&probe); return Ok(c); }
        }
    }
    Err(io::Error::new(io::ErrorKind::PermissionDenied, "aucun dossier de données inscriptible"))
}

fn valid_date(d: Option<&str>) -> Value {
    match d { Some(s) if s.len() == 10 && s.bytes().enumerate().all(|(i, b)| if i == 4 || i == 7 { b == b'-' } else { b.is_ascii_digit() }) => json!(s), _ => Value::Null }
}

impl Store {
    pub fn open(dir: impl AsRef<Path>) -> io::Result<Store> { fs::create_dir_all(dir.as_ref())?; Ok(Store { dir: dir.as_ref().to_path_buf() }) }
    fn file(&self, id: u64) -> PathBuf { self.dir.join(format!("dossier-{id}.json")) }
    fn ids(&self) -> Vec<u64> {
        fs::read_dir(&self.dir).into_iter().flatten().flatten()
            .filter_map(|e| { let n = e.file_name().to_string_lossy().into_owned(); n.strip_prefix("dossier-")?.strip_suffix(".json")?.parse().ok() }).collect()
    }
    fn next_id(&self) -> u64 { self.ids().into_iter().max().unwrap_or(0) + 1 }
    fn read_row(&self, id: u64) -> Option<Value> { serde_json::from_str(&fs::read_to_string(self.file(id)).ok()?).ok() }
    fn write_row(&self, id: u64, row: &Value) -> io::Result<()> {
        let f = self.file(id);
        let tmp = f.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(row)?)?;
        fs::rename(tmp, f)
    }
    fn to_row(id: u64, rec: &Value) -> Result<Value, String> {
        let p = &rec["patient"];
        let (ln, fnm) = (p["last_name"].as_str().unwrap_or("").trim(), p["first_names"].as_str().unwrap_or("").trim());
        if ln.is_empty() || fnm.is_empty() { return Err("nom et prénoms obligatoires".into()); }
        let now = now_iso();
        Ok(json!({"id": id, "last_name": ln.chars().take(190).collect::<String>(), "first_names": fnm.chars().take(190).collect::<String>(),
            "measurement_date": valid_date(rec["measurement_date"].as_str()),
            "scenario": if rec["scenario"] == "developmental" { "developmental" } else { "congenital" },
            "created_at": now, "updated_at": now, "record": rec}))
    }

    pub fn list(&self) -> Vec<Summary> {
        let mut v: Vec<Summary> = self.ids().into_iter().filter_map(|id| {
            let r = self.read_row(id)?;
            Some(Summary { id, last_name: r["last_name"].as_str()?.into(), first_names: r["first_names"].as_str().unwrap_or("").into(),
                measurement_date: r["measurement_date"].as_str().map(String::from), scenario: r["scenario"].as_str().unwrap_or("congenital").into(),
                updated_at: r["updated_at"].as_str().unwrap_or("").into() })
        }).collect();
        v.sort_by(|a, b| {
            let ka = a.measurement_date.clone().unwrap_or_else(|| a.updated_at.clone());
            let kb = b.measurement_date.clone().unwrap_or_else(|| b.updated_at.clone());
            kb.cmp(&ka).then(b.id.cmp(&a.id))
        });
        v
    }
    pub fn save(&self, rec: &Value) -> Result<u64, String> {
        let id = self.next_id();
        let row = Self::to_row(id, rec)?;
        self.write_row(id, &row).map_err(|e| e.to_string())?;
        Ok(id)
    }
    pub fn get(&self, id: u64) -> Result<Value, String> {
        let r = self.read_row(id).ok_or("dossier introuvable")?;
        let mut rec = r["record"].clone();
        if let Some(o) = rec.as_object_mut() { o.insert("id".into(), json!(id)); }
        Ok(rec)
    }
    pub fn delete(&self, id: u64) -> Result<(), String> { fs::remove_file(self.file(id)).map_err(|_| "dossier introuvable".to_string()) }
    pub fn export_all(&self) -> Vec<Value> { self.ids().into_iter().filter_map(|id| self.read_row(id).map(|r| r["record"].clone())).collect() }
    pub fn import_all(&self, recs: &[Value]) -> usize {
        let mut n = 0;
        for rec in recs {
            if rec["input_values"].is_null() { continue; }
            let id = self.next_id();
            if let Ok(row) = Self::to_row(id, rec) { if self.write_row(id, &row).is_ok() { n += 1; } }
        }
        n
    }
}
