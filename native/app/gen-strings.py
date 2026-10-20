#!/usr/bin/env python3
"""Régénère ui/tr.slint et src/tr_gen.rs à partir de strings.json (une propriété Slint par clé)."""
import json, re
S = json.load(open('strings.json', encoding='utf-8'))
keys = list(S['fr'].keys())
assert keys == list(S['en'].keys()) or set(keys) == set(S['en'])
kebab = lambda k: re.sub(r'(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])', '-', k).lower()
sl = '// Généré par gen-strings.py depuis strings.json — ne pas modifier à la main\nexport global T {\n'
for k in keys: sl += f'    in property <string> {kebab(k)};\n'
sl += ('    in property <[string]> sex-options;\n    in property <[string]> basis-options;\n    in property <[string]> scenario-options;\n'
       '    in property <[string]> method-options;\n    in property <[string]> lang-options: ["Français", "English"];\n}\n')
open('ui/tr.slint', 'w', encoding='utf-8').write(sl)
rs = ('// Généré par gen-strings.py — associe chaque clé de strings.json à la propriété Slint correspondante.\n'
      'use crate::{MainWindow, T};\nuse slint::ComponentHandle;\nuse std::collections::HashMap;\n\n'
      'pub fn apply(ui: &MainWindow, m: &HashMap<String, String>) {\n    let t = ui.global::<T>();\n')
for k in keys: rs += f'    t.set_{kebab(k).replace("-", "_")}(m["{k}"].as_str().into());\n'
rs += '}\n'
open('src/tr_gen.rs', 'w', encoding='utf-8').write(rs)
print(len(keys), 'clés')
