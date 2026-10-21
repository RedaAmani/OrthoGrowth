\# OrthoGrowth



Estimates of lower-limb length discrepancy and theoretical epiphysiodesis timing in pediatric orthopedics, using the multiplier method. English and French interface.



> \*\*Clinical disclaimer.\*\* OrthoGrowth is a calculation prototype, not a clinically validated medical device. Its calculations, assumptions and multiplier data require independent verification. Do not use it as the sole basis for diagnosis, treatment planning or epiphysiodesis decisions. Verify all measurements and results with a qualified specialist.



\## Two ways to use it



\### WordPress plugin renamed as "Pediatric Limb Length Calculator" (files at the root of this repository)

Browser-based. Requires WordPress 6.2+ and PHP 8.0+. Records are stored in the WordPress database, per user.



1\. Upload the plugin to `/wp-content/plugins/` (or install the plugin ZIP from the Plugins screen) and activate it.

2\. Open the Pediatric Limb Length Calculator menu in the administration sidebar and grant access to individual users.

3\. Create a page restricted to authorized users and add the shortcode `\[pediatric-limb-length-calculator]`.

4\. Test with fictitious data before any real use.



See \[`readme.txt`](readme.txt) for privacy, data retention and the changelog. The site owner is responsible for access control, secure hosting, HTTPS and compliance with applicable health-data laws.



\### Native application for Windows and Linux (\[`native/`](native/))

Standalone and offline, written in Rust with the Slint interface: a single executable of about 6 MB, no WordPress, no browser, no installation. Records are readable `.json` files stored next to the program.

Download it from the \[Releases](https://github.com/RedaAmani/OrthoGrowth/releases) page. Build instructions are in \[`native/README.md`](native/README.md).



\## Sources and attribution



The multiplier-method approach is based on:



\- Paley D, Bhave A, Herzenberg JE, Bowen JR. Multiplier method for predicting limb-length discrepancy. \*Journal of Bone and Joint Surgery American\*. 2000;82(10):1432–1446. https://pubmed.ncbi.nlm.nih.gov/11057472/

\- Mills G, Nelson S. An improved spreadsheet for calculating limb length discrepancy and epiphysiodesis timing using the multiplier method. \*Journal of Children's Orthopaedics\*. 2016;10:313–319. https://doi.org/10.1007/s11832-016-0754-4



The multiplier table used by both the plugin and the native application is transcribed from the 2016 article, which is licensed under \[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Changes: the values were transcribed into code, and the girls' value at 3.08 years, printed as 3.03 in the source and apparently inconsistent with adjacent rows, is replaced by 2.03 as an inferred correction (not a value printed in the article). This correction must be independently checked before any clinical use.



\## Licenses

\- WordPress plugin: GPL-2.0-or-later.

\- Native application (`native/`): GPL-3.0 (the Slint interface is used under GPL-3.0). See `native/LICENSE`.



\## Support

If this project is useful to you: https://buymeacoffee.com/dramani



\## En français

OrthoGrowth estime l'inégalité de longueur des membres inférieurs et le moment théorique d'une épiphysiodèse par la méthode des multiplicateurs. Il existe sous deux formes : un \*\*plugin WordPress\*\* (fichiers à la racine du dépôt) et une \*\*application autonome hors ligne\*\* pour Windows et Linux (dossier `native/`, exécutables dans l'onglet Releases). Prototype non validé cliniquement : ne pas utiliser seul pour décider d'un traitement, et ne pas saisir de vraies données patient avant validation.

