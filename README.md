# OrthoGrowth
OrthoGrowth is a Wordpress plugin that provides a browser-based calculation interface for estimating lower-limb length discrepancy and theoretical epiphysiodesis timing.


Contributors: doczen

Donate link: https://buymeacoffee.com/dramani

Tags: medical, orthopedics, calculation

Requires at least: 6.2

Tested up to: 7.1

Stable tag: 0.5.0

License: GPLv2 or later

License URI: https://www.gnu.org/licenses/gpl-2.0.html

A browser-based prototype for estimating limb-length discrepancy and theoretical epiphysiodesis timing in pediatric orthopedics.

== Description ==

OrthoGrowth is a browser-based calculation prototype intended for use in pediatric orthopedics. It estimates lower-limb length discrepancy and theoretical epiphysiodesis timing using the multiplier method. The interface supports English and French.

Authorized users can create and manage their own records. Site administrators can view all records and manage user access. Records include patient names, measurement dates, entered measurements and calculation results, and are stored in the WordPress database.

**Clinical disclaimer:** OrthoGrowth is a calculation prototype, not a clinically validated medical device. Its calculations, assumptions, multiplier data and inferred correction described below require independent verification. Do not use it as the sole basis for diagnosis, treatment planning or epiphysiodesis decisions. Verify all measurements and results with a qualified specialist.

The site owner is responsible for access control, secure hosting, HTTPS, protected backups, appropriate retention, and compliance with applicable privacy and health-data laws. Do not enter real patient data into a test or publicly accessible installation.

== Installation ==

1. Upload the `orthogrowth` folder to `/wp-content/plugins/`, or install the plugin ZIP through the WordPress Plugins screen.
2. Activate OrthoGrowth. On multisite installations, network activation creates a separate records table for each site, including newly created sites.
3. Open the OrthoGrowth menu in the WordPress administration sidebar and grant access to individual users.
4. Create a page available only to authorized users and add the shortcode `[orthogrowth]`.
5. Test calculation, saving, loading, exporting and deletion with fictitious data before any real use.

== Privacy and data retention ==

OrthoGrowth stores patient names, measurement dates, measurements and calculation results. Authorized non-administrator users can access only records they created; site administrators can access all records. The plugin registers WordPress personal-data exporter and eraser callbacks for records associated with a WordPress account. Erasing an account's OrthoGrowth data permanently deletes records created by that account.

By default, uninstalling the plugin preserves saved records. An administrator can explicitly enable permanent deletion on uninstall in the OrthoGrowth administration menu. If enabled, uninstalling deletes records for the relevant site; with network activation, the setting is evaluated separately for each site. Keep secure backups and establish a retention policy appropriate to your jurisdiction.

== Frequently Asked Questions ==

= Does OrthoGrowth provide a clinical diagnosis? =
No. It provides estimates only. It is not a validated medical device and must not be used alone to make treatment decisions.

= Who can view saved records? =
Authorized users can view their own records. Site administrators can view all records.

= Can I export or erase records associated with a WordPress account? =
Yes. The plugin provides callbacks for the WordPress personal-data export and erasure tools. The eraser permanently deletes OrthoGrowth records created by the account being processed.

= What happens when the plugin is uninstalled? =
Records are preserved by default. Permanent deletion occurs only if an administrator enabled the corresponding option before uninstalling. On multisite, this setting is applied per site.

== Sources and attribution ==

The multiplier-method approach is based on published orthopedic literature, including:

- Paley D, Bhave A, Herzenberg JE, Bowen JR. Multiplier method for predicting limb-length discrepancy. *Journal of Bone and Joint Surgery American*. 2000;82(10):1432–1446. https://pubmed.ncbi.nlm.nih.gov/11057472/
- Mills G, Nelson S. An improved spreadsheet for calculating limb length discrepancy and epiphysiodesis timing using the multiplier method. *Journal of Children's Orthopaedics*. 2016;10:313–319. https://doi.org/10.1007/s11832-016-0754-4

The 2016 article is licensed under Creative Commons Attribution 4.0 International (CC BY 4.0): https://creativecommons.org/licenses/by/4.0/. Article-derived material must be attributed to the authors and source, linked to the license, and any changes disclosed. The source table's value for girls at 3.08 years is printed as 3.03, which appears inconsistent with adjacent rows. OrthoGrowth uses 2.03 as an inferred correction, not as a value explicitly printed in the article. This correction must be independently checked before any clinical use.
