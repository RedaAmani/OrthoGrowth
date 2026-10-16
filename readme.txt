=== OrthoGrowth ===
Tags: medical, orthopedics, calculation
Requires at least: 6.2
Requires PHP: 8.0
Stable tag: 0.4.1
License: GPLv2 or later
License URI: https://www.gnu.org/licenses/gpl-2.0.html

A browser-based prototype for estimating lower-limb length discrepancy and theoretical epiphysiodesis timing using the multiplier method. It is not clinically validated and must not be used alone to make treatment decisions.

== Description ==

OrthoGrowth provides a browser-based calculation interface for estimating lower-limb length discrepancy and theoretical epiphysiodesis timing. Authorized users can save their own records. Site administrators can view all records and manage user access.

The plugin stores patient names, measurement dates, entered measurements and calculation results in the WordPress database. The site owner is responsible for access control, secure hosting, backups, retention, and compliance with applicable privacy and health-data laws.

**Clinical disclaimer:** This is a calculation prototype, not a validated medical device. Do not use it alone to decide treatment or epiphysiodesis. Verify measurements, assumptions and results with a specialist.

== Installation ==

1. Install and activate the plugin from the WordPress admin area.
2. Open the OrthoGrowth menu in the WordPress administration sidebar and grant access to individual users.
3. Create a page available only to authorized users and add the shortcode `[orthogrowth]`.
4. Test calculation, saving, loading and deletion with fictitious data before any real use.

== Privacy and data retention ==

Saved records contain patient-identifiable and clinical information. Each authorized non-administrator can access only records they created. Administrators can access all records. Use HTTPS, individual accounts, least-privilege access, protected backups and a retention policy appropriate to your jurisdiction.

By default, uninstalling the plugin preserves records in the database. An administrator can explicitly enable permanent deletion on uninstall in the OrthoGrowth administration menu. If enabled, uninstalling deletes all OrthoGrowth records irreversibly.

== Frequently Asked Questions ==

= Does this plugin provide a clinical diagnosis? =
No. It provides estimates only. The calculations and assumptions require independent clinical verification.

= Who can view saved records? =
Users with OrthoGrowth access can view their own records. Site administrators can view all records.

= What happens when the plugin is uninstalled? =
Records are preserved by default. Permanent deletion occurs only if an administrator enabled the corresponding option before uninstalling.

== Sources and attribution ==

The multiplier-method approach is based on published orthopedic literature, including:

- Paley D, Bhave A, Herzenberg JE, Bowen JR. Multiplier method for predicting limb-length discrepancy. *Journal of Bone and Joint Surgery American*. 2000;82(10):1432–1446. https://pubmed.ncbi.nlm.nih.gov/11057472/
- Mills KA, Nelson SC. An improved spreadsheet for calculating limb length discrepancy and epiphysiodesis timing using the multiplier method. *Journal of Children's Orthopaedics*. 2016. https://doi.org/10.1007/s11832-016-0754-4

The 2016 article is licensed under Creative Commons Attribution 4.0 International (CC BY 4.0): https://creativecommons.org/licenses/by/4.0/. Article-derived material must be attributed to the authors and source, linked to the license, and any changes disclosed. Multiplier data and any inferred corrections should be independently checked before clinical use. See the source article and its license for details.

== Changelog ==

= 0.4.1 =
* Remove LISEZ-MOI.txt and provide complete English and French documentation in readme.txt.
* Move OrthoGrowth access and settings to a top-level administration menu.

= 0.4.0 =
* Remove obsolete compatibility and migration code.
* Prepare public-directory structure and documentation.
* Restrict non-administrator access to records created by the current user.
* Add opt-in permanent data deletion on uninstall, preserving data by default.
* Enqueue JavaScript and CSS through WordPress APIs.
* Add privacy-policy text and normalize version metadata.
* Place OrthoGrowth settings in a top-level administration menu.

== Documentation française ==

OrthoGrowth est un prototype dans le navigateur destiné à estimer l'inégalité de longueur des membres inférieurs et le calendrier théorique d'une épiphysiodèse à l'aide de la méthode des multiplicateurs. Il n'a pas été validé cliniquement et ne doit pas être utilisé seul pour prendre des décisions thérapeutiques.

== Description ==

OrthoGrowth propose une interface de calcul dans le navigateur pour estimer l'inégalité de longueur des membres inférieurs et le calendrier théorique d'une épiphysiodèse. Les utilisateurs autorisés peuvent enregistrer leurs propres dossiers. Les administrateurs du site peuvent consulter tous les dossiers et gérer les accès des utilisateurs.

L'extension enregistre dans la base de données WordPress les noms des patients, les dates de mesure, les mesures saisies et les résultats des calculs. Le propriétaire du site est responsable du contrôle des accès, de la sécurité de l'hébergement, des sauvegardes, de la durée de conservation et du respect des lois applicables à la confidentialité et aux données de santé.

**Avertissement clinique :** il s'agit d'un prototype de calcul, et non d'un dispositif médical validé. Ne l'utilisez pas seul pour décider d'un traitement ou d'une épiphysiodèse. Vérifiez les mesures, les hypothèses et les résultats avec un spécialiste.

== Installation ==

1. Installez et activez l'extension depuis l'administration WordPress.
2. Ouvrez le menu OrthoGrowth dans la barre latérale de l'administration WordPress et accordez l'accès aux utilisateurs concernés.
3. Créez une page réservée aux utilisateurs autorisés et ajoutez-y le shortcode `[orthogrowth]`.
4. Avant toute utilisation réelle, testez les calculs, l'enregistrement, le chargement et la suppression avec des données fictives.

== Confidentialité et conservation des données ==

Les dossiers enregistrés contiennent des informations cliniques et des données permettant d'identifier les patients. Chaque utilisateur non administrateur autorisé ne peut accéder qu'aux dossiers qu'il a créés. Les administrateurs peuvent accéder à tous les dossiers. Utilisez HTTPS, des comptes individuels, des droits d'accès limités au nécessaire, des sauvegardes protégées et une politique de conservation adaptée à votre juridiction.

Par défaut, la désinstallation de l'extension conserve les dossiers dans la base de données. Un administrateur peut activer explicitement la suppression définitive lors de la désinstallation dans le menu d'administration OrthoGrowth. Si cette option est activée, la désinstallation supprime irréversiblement tous les dossiers OrthoGrowth.

== Foire aux questions ==

= Cette extension fournit-elle un diagnostic clinique ? =
Non. Elle fournit uniquement des estimations. Les calculs et les hypothèses doivent faire l'objet d'une vérification clinique indépendante.

= Qui peut consulter les dossiers enregistrés ? =
Les utilisateurs autorisés par OrthoGrowth peuvent consulter leurs propres dossiers. Les administrateurs du site peuvent consulter tous les dossiers.

= Que se passe-t-il lorsque l'extension est désinstallée ? =
Les dossiers sont conservés par défaut. Ils ne sont supprimés définitivement que si un administrateur a activé l'option correspondante avant la désinstallation.

== Sources et attribution ==

La méthode des multiplicateurs s'appuie notamment sur les publications orthopédiques suivantes :

- Paley D, Bhave A, Herzenberg JE, Bowen JR. *Multiplier method for predicting limb-length discrepancy*. Journal of Bone and Joint Surgery American. 2000;82(10):1432–1446. https://pubmed.ncbi.nlm.nih.gov/11057472/
- Mills KA, Nelson SC. *An improved spreadsheet for calculating limb length discrepancy and epiphysiodesis timing using the multiplier method*. Journal of Children's Orthopaedics. 2016. https://doi.org/10.1007/s11832-016-0754-4

L'article de 2016 est publié sous licence Creative Commons Attribution 4.0 International (CC BY 4.0) : https://creativecommons.org/licenses/by/4.0/. Tout contenu dérivé de cet article doit créditer les auteurs et la source, fournir un lien vers la licence et signaler les modifications apportées. Les données de multiplicateurs et toute correction déduite doivent être vérifiées indépendamment avant toute utilisation clinique. Consultez l'article source et sa licence pour plus de détails.

== Journal des modifications ==

= 0.4.1 =
* Suppression de LISEZ-MOI.txt et regroupement de la documentation anglaise et française dans readme.txt.
* Déplacement des accès et réglages OrthoGrowth dans un menu principal de l’administration.

= 0.4.0 =
* Suppression du code obsolète de compatibilité et de migration.
* Préparation de la structure et de la documentation pour une publication publique.
* Limitation de l'accès des non-administrateurs aux dossiers qu'ils ont créés.
* Ajout d'une option de suppression définitive à la désinstallation, les données étant conservées par défaut.
* Chargement de JavaScript et du CSS avec les API WordPress.
* Ajout du texte relatif à la politique de confidentialité et harmonisation des métadonnées de version.
* Placement des réglages OrthoGrowth dans un menu principal de l'administration.
