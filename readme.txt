=== Pediatric Limb Length Calculator ===
Contributors: doczen
Donate link: https://buymeacoffee.com/dramani
Tags: medical, orthopedics, calculation
Requires at least: 6.2
Tested up to: 7.1
Stable tag: 0.6.0
License: GPLv2 or later
License URI: https://www.gnu.org/licenses/gpl-2.0.html

A browser-based prototype for estimating limb-length discrepancy and theoretical epiphysiodesis timing in pediatric orthopedics.

== Description ==

Pediatric Limb Length Calculator is a browser-based calculation prototype intended for use in pediatric orthopedics. It estimates lower-limb length discrepancy and theoretical epiphysiodesis timing using the multiplier method. The interface supports English and French.

Authorized users can create and manage their own records. Site administrators can view all records and manage user access. Records include patient names, measurement dates, entered measurements and calculation results, and are stored in the WordPress database.

**Clinical disclaimer:** Pediatric Limb Length Calculator is a calculation prototype, not a clinically validated medical device. Its calculations, assumptions, multiplier data and inferred correction described below require independent verification. Do not use it as the sole basis for diagnosis, treatment planning or epiphysiodesis decisions. Verify all measurements and results with a qualified specialist.

The site owner is responsible for access control, secure hosting, HTTPS, protected backups, appropriate retention, and compliance with applicable privacy and health-data laws. Do not enter real patient data into a test or publicly accessible installation.

== Installation ==

1. Upload the `pediatric-limb-length-calculator` folder to `/wp-content/plugins/`, or install the plugin ZIP through the WordPress Plugins screen.
2. Activate Pediatric Limb Length Calculator. On multisite installations, network activation creates a separate records table for each site, including newly created sites.
3. Open the Pediatric Limb Length Calculator menu in the WordPress administration sidebar and grant access to individual users.
4. Create a page available only to authorized users and add the shortcode `[pediatric-limb-length-calculator]`.
5. Test calculation, saving, loading, exporting and deletion with fictitious data before any real use.

== Privacy and data retention ==

Pediatric Limb Length Calculator stores patient names, measurement dates, measurements and calculation results. Authorized non-administrator users can access only records they created; site administrators can access all records. The plugin registers WordPress personal-data exporter and eraser callbacks for records associated with a WordPress account. Erasing an account's Pediatric Limb Length Calculator data permanently deletes records created by that account.

By default, uninstalling the plugin preserves saved records. An administrator can explicitly enable permanent deletion on uninstall in the Pediatric Limb Length Calculator administration menu. If enabled, uninstalling deletes records for the relevant site; with network activation, the setting is evaluated separately for each site. Keep secure backups and establish a retention policy appropriate to your jurisdiction.

== Frequently Asked Questions ==

= Does Pediatric Limb Length Calculator provide a clinical diagnosis? =
No. It provides estimates only. It is not a validated medical device and must not be used alone to make treatment decisions.

= Who can view saved records? =
Authorized users can view their own records. Site administrators can view all records.

= Can I export or erase records associated with a WordPress account? =
Yes. The plugin provides callbacks for the WordPress personal-data export and erasure tools. The eraser permanently deletes Pediatric Limb Length Calculator records created by the account being processed.

= What happens when the plugin is uninstalled? =
Records are preserved by default. Permanent deletion occurs only if an administrator enabled the corresponding option before uninstalling. On multisite, this setting is applied per site.

== Sources and attribution ==

The multiplier-method approach is based on published orthopedic literature, including:

- Paley D, Bhave A, Herzenberg JE, Bowen JR. Multiplier method for predicting limb-length discrepancy. *Journal of Bone and Joint Surgery American*. 2000;82(10):1432–1446. https://pubmed.ncbi.nlm.nih.gov/11057472/
- Mills G, Nelson S. An improved spreadsheet for calculating limb length discrepancy and epiphysiodesis timing using the multiplier method. *Journal of Children's Orthopaedics*. 2016;10:313–319. https://doi.org/10.1007/s11832-016-0754-4

The 2016 article is licensed under Creative Commons Attribution 4.0 International (CC BY 4.0): https://creativecommons.org/licenses/by/4.0/. Article-derived material must be attributed to the authors and source, linked to the license, and any changes disclosed. The source table's value for girls at 3.08 years is printed as 3.03, which appears inconsistent with adjacent rows. Pediatric Limb Length Calculator uses 2.03 as an inferred correction, not as a value explicitly printed in the article. This correction must be independently checked before any clinical use.

== Upgrade Notice ==

= 0.6.0 =
Renames the plugin and its WordPress-specific identifiers to Pediatric Limb Length Calculator.


= 0.5.1 =
This maintenance release clarifies security checks and documents intentional direct database operations. Test with fictitious data before use.

= 0.5.0 =
This release updates the readme, adds WordPress personal-data export and erasure support, improves record payload validation, and adds schema checks and multisite table setup. Test with fictitious data before use.

== Changelog ==

= 0.6.0 =
* Rename the plugin, text domain, shortcode, hooks, options, database table, assets, and related identifiers to Pediatric Limb Length Calculator.


= 0.5.1 =
* Use WordPress's AJAX nonce verification helper.
* Validate the administration status query parameter before displaying a notice.
* Add targeted explanations for custom-table database queries and the intentional absence of record caching.
* Prefix multisite uninstall variables and document the opt-in table drop.

= 0.5.0 =
* Use an English-only readme with corrected WordPress.org metadata and a concise short description.
* Add the project URL and donation link to the plugin documentation.
* Correct the Mills and Nelson citation.
* Add WordPress personal-data export and erasure callbacks.
* Validate and sanitize saved JSON payloads using an allowlist of supported fields.
* Add schema version checks and multisite table setup, including newly created sites.
* Remove the unused Domain Path header and make the HTML template's default text English.

= 0.4.1 =
* Move Pediatric Limb Length Calculator access and settings to a top-level administration menu.
* Remove the separate LISEZ-MOI.txt file.

= 0.4.0 =
* Restrict non-administrator access to records created by the current user.
* Add opt-in permanent data deletion on uninstall, preserving data by default.
* Enqueue JavaScript and CSS through WordPress APIs.
* Add privacy-policy text and normalize version metadata.
