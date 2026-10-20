# OrthoGrowth natif (Windows et Linux)

Version autonome et hors ligne d'OrthoGrowth, écrite en Rust avec l'interface Slint : un seul exécutable
(environ 6 Mo sous Windows), sans navigateur embarqué ni installation. Les dossiers sont des fichiers
`donnees/dossier-N.json` lisibles, compatibles avec ceux des autres versions.

**Prototype non validé cliniquement.** Voir l'avertissement affiché dans l'application et le readme du plugin.

## Télécharger
Page **Releases** du dépôt : `OrthoGrowth-<version>-windows-x64.zip` ou `-linux-x64.zip`.

## Contenu de ce dossier
- `core/` : noyau de calcul (multiplicateurs de Mills et Nelson, modèles congénital et développemental,
  épiphysiodèse théorique) et stockage en fichiers. Port de la logique JavaScript du plugin ; tests : `cargo test`.
- `app/`  : interface (`app/ui/app.slint`), logique (`app/src/lib.rs`), textes FR/EN (`app/strings.json`,
  puis `python3 app/gen-strings.py` pour régénérer `tr.slint` et `tr_gen.rs`), icône (`app/assets/`),
  informations de version Windows (`app/app.rc`).
- `dist-doc/` : documentation livrée avec les exécutables ; `compilation-windows-depuis-linux.md` : méthode de compilation croisée.

## Compiler soi-même (Rust 1.89 ou plus récent ; Linux : paquet `libfontconfig1-dev`)

    cd native
    cargo test --locked --release -p orthogrowth-core
    cargo build --locked --release -p orthogrowth      # -> target/release/OrthoGrowth(.exe)

`Cargo.lock` fige Slint 1.16.1, qui compile avec Rust 1.89 ou plus récent.

## Publier une version
Onglet **Actions** > « Compiler OrthoGrowth natif » > *Run workflow* pour obtenir les exécutables en artefacts,
ou créer un tag pour publier une release :

    git tag native-v0.5.1 && git push origin native-v0.5.1

## Licence
Ce dossier est sous **GPL-3.0** (voir `LICENSE`), car l'interface Slint est utilisée sous GPL-3.0.
Le plugin WordPress, à la racine du dépôt, reste sous sa propre licence (GPLv2 ou ultérieure).
