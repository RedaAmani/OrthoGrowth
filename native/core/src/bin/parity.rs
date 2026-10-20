// Lit des cas {"lang":..,"inputs":{..}} (un par ligne) et écrit le résultat au même format que l'interface JS.
use orthogrowth_core::{calculate, format::display, Inputs};
use serde_json::{json, Value};
use std::io::BufRead;
fn main() {
    for line in std::io::stdin().lock().lines() {
        let v: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let lang = v["lang"].as_str().unwrap_or("en").to_string();
        let inp: Inputs = serde_json::from_value(v["inputs"].clone()).unwrap();
        let out = match calculate(&inp) {
            Ok(o) => json!({"ok": true, "display": serde_json::to_value(display(&o, &lang)).unwrap()}),
            Err(e) => json!({"ok": false, "error": e.key()}),
        };
        println!("{}", out);
    }
}
