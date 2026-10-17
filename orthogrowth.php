<?php
/**
 * Plugin Name: OrthoGrowth
 * Description: Browser-based estimates of limb-length discrepancy and theoretical epiphysiodesis timing.
 * Plugin URI: https://github.com/RedaAmani/OrthoGrowth
 * Version: 0.5.1
 * Requires at least: 6.2
 * Requires PHP: 8.0
 * Author: Dr M.R. Amani
 * License: GPL-2.0-or-later
 * License URI: https://www.gnu.org/licenses/gpl-2.0.html
 * Text Domain: orthogrowth
 */

if (!defined('ABSPATH')) { exit; }

final class OrthoGrowth {
    const VERSION = '0.5.1';
    const DB_VERSION = '1.0';
    const CAPABILITY = 'orthogrowth_use';
    const NONCE_ACTION = 'orthogrowth_server_action';
    const DELETE_OPTION = 'orthogrowth_delete_data_on_uninstall';

    public static function init() {
        register_activation_hook(__FILE__, [__CLASS__, 'activate']);
        add_action('plugins_loaded', [__CLASS__, 'maybe_upgrade_schema']);
        add_action('wp_initialize_site', [__CLASS__, 'initialize_new_site'], 10, 1);
        add_shortcode('orthogrowth', [__CLASS__, 'shortcode']);
        add_action('wp_enqueue_scripts', [__CLASS__, 'enqueue_frontend_assets']);
        add_action('admin_menu', [__CLASS__, 'admin_menu']);
        add_action('admin_post_orthogrowth_access', [__CLASS__, 'save_access']);
        add_action('admin_post_orthogrowth_settings', [__CLASS__, 'save_settings']);
        add_action('wp_ajax_orthogrowth_list', [__CLASS__, 'ajax_list']);
        add_action('wp_ajax_orthogrowth_save', [__CLASS__, 'ajax_save']);
        add_action('wp_ajax_orthogrowth_get', [__CLASS__, 'ajax_get']);
        add_action('wp_ajax_orthogrowth_delete', [__CLASS__, 'ajax_delete']);
        add_action('admin_init', [__CLASS__, 'register_privacy_policy_content']);
        add_filter('wp_privacy_personal_data_exporters', [__CLASS__, 'register_privacy_exporter']);
        add_filter('wp_privacy_personal_data_erasers', [__CLASS__, 'register_privacy_eraser']);
    }

    public static function table_name() {
        global $wpdb;
        return $wpdb->prefix . 'orthogrowth_records';
    }

    public static function activate($network_wide = false) {
        if (is_multisite() && $network_wide) {
            update_site_option('orthogrowth_network_active', 1);
            $sites = get_sites(['number' => 0, 'fields' => 'ids']);
            foreach ($sites as $site_id) {
                switch_to_blog((int) $site_id);
                self::install_site();
                restore_current_blog();
            }
            return;
        }
        self::install_site();
    }

    public static function initialize_new_site($new_site) {
        if (!is_multisite() || !get_site_option('orthogrowth_network_active', 0)) { return; }
        $blog_id = is_object($new_site) && isset($new_site->blog_id) ? (int) $new_site->blog_id : (int) $new_site;
        if ($blog_id < 1) { return; }
        switch_to_blog($blog_id);
        self::install_site();
        restore_current_blog();
    }

    private static function install_site() {
        global $wpdb;
        require_once ABSPATH . 'wp-admin/includes/upgrade.php';
        $table = self::table_name();
        $charset = $wpdb->get_charset_collate();
        $sql = "CREATE TABLE {$table} (
            id bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            last_name varchar(190) NOT NULL,
            first_names varchar(190) NOT NULL,
            measurement_date date DEFAULT NULL,
            scenario varchar(32) NOT NULL DEFAULT 'congenital',
            record_json longtext NOT NULL,
            created_by bigint(20) unsigned NOT NULL,
            updated_by bigint(20) unsigned NOT NULL,
            created_at datetime NOT NULL,
            updated_at datetime NOT NULL,
            PRIMARY KEY  (id),
            KEY patient_name (last_name, first_names),
            KEY measurement_date (measurement_date),
            KEY created_by (created_by)
        ) {$charset};";
        dbDelta($sql);
        $role = get_role('administrator');
        if ($role && !$role->has_cap(self::CAPABILITY)) { $role->add_cap(self::CAPABILITY); }
        update_option('orthogrowth_db_version', self::DB_VERSION, false);
        if (get_option(self::DELETE_OPTION, null) === null) { add_option(self::DELETE_OPTION, 0, '', false); }
    }

    public static function maybe_upgrade_schema() {
        if (get_option('orthogrowth_db_version') !== self::DB_VERSION) {
            self::install_site();
        }
    }

    private static function can_use() {
        return is_user_logged_in() && current_user_can(self::CAPABILITY);
    }

    private static function is_admin_user() { return current_user_can('manage_options'); }

    public static function enqueue_frontend_assets() {
        global $post;
        if (!($post instanceof WP_Post) || !has_shortcode($post->post_content, 'orthogrowth')) { return; }
        self::enqueue_assets();
    }

    private static function enqueue_assets() {
        wp_enqueue_style('orthogrowth-app', plugins_url('orthogrowth.css', __FILE__), [], self::VERSION);
        wp_enqueue_script('orthogrowth-app', plugins_url('orthogrowth.js', __FILE__), [], self::VERSION, true);
        $locale = determine_locale();
        $initial_language = (strpos(strtolower($locale), 'fr') === 0) ? 'fr' : 'en';
        wp_localize_script('orthogrowth-app', 'OrthoGrowthConfig', [
            'ajaxUrl' => admin_url('admin-ajax.php'),
            'nonce' => wp_create_nonce(self::NONCE_ACTION),
            'initialLanguage' => $initial_language,
        ]);
    }

    public static function shortcode() {
        if (!is_user_logged_in()) {
            $url = wp_login_url(get_permalink());
            return '<div class="orthogrowth-access-message"><p>' . esc_html__('This application is restricted to authorized users.', 'orthogrowth') . '</p><p><a href="' . esc_url($url) . '">' . esc_html__('Log in', 'orthogrowth') . '</a></p></div>';
        }
        if (!self::can_use()) {
            return '<div class="orthogrowth-access-message"><p>' . esc_html__('Your account is not authorized to use OrthoGrowth. Contact the site administrator.', 'orthogrowth') . '</p></div>';
        }
        $path = plugin_dir_path(__FILE__) . 'app-template.html';
        if (!is_readable($path)) { return '<p>' . esc_html__('The OrthoGrowth interface template is missing. Reinstall the plugin.', 'orthogrowth') . '</p>'; }
        $template = file_get_contents($path);
        if ($template === false) { return '<p>' . esc_html__('The OrthoGrowth interface could not be loaded.', 'orthogrowth') . '</p>'; }
        self::enqueue_assets();
        return $template;
    }

    public static function admin_menu() {
        add_menu_page(__('OrthoGrowth', 'orthogrowth'), __('OrthoGrowth', 'orthogrowth'), 'manage_options', 'orthogrowth-access', [__CLASS__, 'access_page'], 'dashicons-analytics', 58);
    }

    public static function access_page() {
        if (!current_user_can('manage_options')) { return; }
        $users = get_users(['orderby' => 'display_name', 'order' => 'ASC']);
        echo '<div class="wrap"><h1>' . esc_html__('OrthoGrowth access', 'orthogrowth') . '</h1>';
        echo '<p>' . esc_html__('Authorized users can view and delete only the records they created. Administrators can view and delete all records.', 'orthogrowth') . '</p>';
        // phpcs:ignore WordPress.Security.NonceVerification.Recommended -- Read-only status flag for the redirect notice; it does not trigger a state change.
        if (isset($_GET['updated']) && '1' === sanitize_text_field(wp_unslash($_GET['updated']))) { echo '<div class="notice notice-success is-dismissible"><p>' . esc_html__('Settings saved.', 'orthogrowth') . '</p></div>'; }
        echo '<form method="post" action="' . esc_url(admin_url('admin-post.php')) . '"><input type="hidden" name="action" value="orthogrowth_access">';
        wp_nonce_field('orthogrowth_access_save');
        echo '<table class="widefat striped"><thead><tr><th>' . esc_html__('Authorized', 'orthogrowth') . '</th><th>' . esc_html__('Display name', 'orthogrowth') . '</th><th>' . esc_html__('Username', 'orthogrowth') . '</th><th>' . esc_html__('Email', 'orthogrowth') . '</th><th>' . esc_html__('Role', 'orthogrowth') . '</th></tr></thead><tbody>';
        foreach ($users as $user) {
            $is_admin = user_can($user, 'manage_options');
            $checked = $is_admin || user_can($user, self::CAPABILITY);
            echo '<tr><td><input type="checkbox" name="allowed_users[]" value="' . esc_attr($user->ID) . '" ' . checked($checked, true, false) . ($is_admin ? ' disabled' : '') . '></td>';
            echo '<td>' . esc_html($user->display_name) . '</td><td>' . esc_html($user->user_login) . '</td><td>' . esc_html($user->user_email) . '</td><td>' . esc_html(implode(', ', $user->roles)) . ($is_admin ? ' (' . esc_html__('administrator, access retained', 'orthogrowth') . ')' : '') . '</td></tr>';
        }
        echo '</tbody></table><p><button class="button button-primary" type="submit">' . esc_html__('Save access permissions', 'orthogrowth') . '</button></p></form>';
        echo '<hr><h2>' . esc_html__('Data deletion on uninstall', 'orthogrowth') . '</h2>';
        echo '<p>' . esc_html__('By default, uninstalling OrthoGrowth preserves saved records. You may explicitly choose to permanently delete all OrthoGrowth records when the plugin is uninstalled. This deletion is irreversible.', 'orthogrowth') . '</p>';
        echo '<form method="post" action="' . esc_url(admin_url('admin-post.php')) . '"><input type="hidden" name="action" value="orthogrowth_settings">';
        wp_nonce_field('orthogrowth_settings_save');
        echo '<label><input type="checkbox" name="delete_data_on_uninstall" value="1" ' . checked((bool)get_option(self::DELETE_OPTION, 0), true, false) . '> ' . esc_html__('Permanently delete all saved records when OrthoGrowth is uninstalled', 'orthogrowth') . '</label><p><button class="button button-primary" type="submit">' . esc_html__('Save data deletion setting', 'orthogrowth') . '</button></p></form></div>';
    }

    public static function save_access() {
        if (!current_user_can('manage_options')) { wp_die(esc_html__('Access denied.', 'orthogrowth'), 403); }
        check_admin_referer('orthogrowth_access_save');
        $allowed = isset($_POST['allowed_users']) ? array_map('absint', (array)wp_unslash($_POST['allowed_users'])) : [];
        $admins = array_map('intval', get_users(['role' => 'administrator', 'fields' => 'ID']));
        foreach (get_users() as $user) {
            $id = (int)$user->ID;
            if (in_array($id, $admins, true)) { continue; }
            $wp_user = new WP_User($id);
            if (in_array($id, $allowed, true)) { $wp_user->add_cap(self::CAPABILITY); }
            else { $wp_user->remove_cap(self::CAPABILITY); }
        }
        wp_safe_redirect(add_query_arg(['page' => 'orthogrowth-access', 'updated' => '1'], admin_url('admin.php'))); exit;
    }

    public static function save_settings() {
        if (!current_user_can('manage_options')) { wp_die(esc_html__('Access denied.', 'orthogrowth'), 403); }
        check_admin_referer('orthogrowth_settings_save');
        update_option(self::DELETE_OPTION, isset($_POST['delete_data_on_uninstall']) ? 1 : 0, false);
        wp_safe_redirect(add_query_arg(['page' => 'orthogrowth-access', 'updated' => '1'], admin_url('admin.php'))); exit;
    }

    public static function register_privacy_policy_content() {
        if (function_exists('wp_add_privacy_policy_content')) {
            $content = '<p>' . esc_html__('OrthoGrowth stores patient names, measurement dates, entered measurements and calculation results in the WordPress database when a user saves a record. Records are accessible to the user who created them and to site administrators. The WordPress personal-data export and erasure tools can export or delete records associated with a user account. The site owner is responsible for setting an appropriate retention period, protecting backups and complying with applicable health-data and privacy laws.', 'orthogrowth') . '</p>';
            wp_add_privacy_policy_content('OrthoGrowth', $content);
        }
    }

    public static function register_privacy_exporter($exporters) {
        $exporters['orthogrowth'] = [
            'exporter_friendly_name' => __('OrthoGrowth records', 'orthogrowth'),
            'callback' => [__CLASS__, 'privacy_exporter'],
        ];
        return $exporters;
    }

    public static function privacy_exporter($email_address, $page = 1) {
        $user = get_user_by('email', $email_address);
        if (!$user) { return ['data' => [], 'done' => true]; }
        global $wpdb;
        $limit = 100;
        $offset = max(0, (absint($page) - 1) * $limit);
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- This paginated query reads OrthoGrowth's custom table; sensitive records are deliberately not cached.
        $rows = $wpdb->get_results($wpdb->prepare(
            'SELECT id, last_name, first_names, measurement_date, scenario, record_json, created_at, updated_at FROM %i WHERE created_by = %d ORDER BY id ASC LIMIT %d OFFSET %d',
            self::table_name(), (int) $user->ID, $limit, $offset
        ), ARRAY_A);
        if (!is_array($rows)) { return ['data' => [], 'done' => true]; }
        $items = [];
        foreach ($rows as $row) {
            $items[] = [
                'group_id' => 'orthogrowth-records',
                'group_label' => __('OrthoGrowth records', 'orthogrowth'),
                'item_id' => 'orthogrowth-record-' . (int) $row['id'],
                'data' => [
                    ['name' => __('Patient last name', 'orthogrowth'), 'value' => $row['last_name']],
                    ['name' => __('Patient first name(s)', 'orthogrowth'), 'value' => $row['first_names']],
                    ['name' => __('Measurement date', 'orthogrowth'), 'value' => $row['measurement_date'] ?? ''],
                    ['name' => __('Scenario', 'orthogrowth'), 'value' => $row['scenario']],
                    ['name' => __('Record data (JSON)', 'orthogrowth'), 'value' => $row['record_json']],
                    ['name' => __('Record created (UTC)', 'orthogrowth'), 'value' => $row['created_at']],
                    ['name' => __('Record updated (UTC)', 'orthogrowth'), 'value' => $row['updated_at']],
                ],
            ];
        }
        return ['data' => $items, 'done' => count($rows) < $limit];
    }

    public static function register_privacy_eraser($erasers) {
        $erasers['orthogrowth'] = [
            'eraser_friendly_name' => __('OrthoGrowth records', 'orthogrowth'),
            'callback' => [__CLASS__, 'privacy_eraser'],
        ];
        return $erasers;
    }

    public static function privacy_eraser($email_address, $page = 1) {
        $user = get_user_by('email', $email_address);
        if (!$user) {
            return ['items_removed' => false, 'items_retained' => false, 'messages' => [], 'done' => true];
        }
        global $wpdb;
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Deletes the requesting account's records from the custom table; no cache is maintained for these records.
        $deleted = $wpdb->delete(self::table_name(), ['created_by' => (int) $user->ID], ['%d']);
        if ($deleted === false) {
            return ['items_removed' => false, 'items_retained' => true, 'messages' => [__('OrthoGrowth records could not be deleted. The site administrator should investigate the database error.', 'orthogrowth')], 'done' => true];
        }
        return ['items_removed' => $deleted > 0, 'items_retained' => false, 'messages' => [], 'done' => true];
    }

    private static function verify_ajax() {
        if (!is_user_logged_in()) { wp_send_json_error(['message' => __('Session expired. Please log in again.', 'orthogrowth')], 401); }
        if (!self::can_use()) { wp_send_json_error(['message' => __('Access denied.', 'orthogrowth')], 403); }
        // Use WordPress' AJAX nonce verifier so the check is explicit and recognized by security tooling.
        if (false === check_ajax_referer(self::NONCE_ACTION, 'nonce', false)) {
            wp_send_json_error(['message' => __('Security token expired. Reload the page.', 'orthogrowth')], 403);
        }
    }

    public static function ajax_list() {
        self::verify_ajax(); global $wpdb; $table = self::table_name();
        if (self::is_admin_user()) {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Required query against the custom records table; sensitive records are deliberately not cached.
            $rows = $wpdb->get_results($wpdb->prepare('SELECT id, last_name, first_names, measurement_date, scenario, updated_at FROM %i ORDER BY COALESCE(measurement_date, DATE(updated_at)) DESC, id DESC LIMIT 2000', $table), ARRAY_A);
        } else {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- User-scoped query against the custom records table; sensitive records are deliberately not cached.
            $rows = $wpdb->get_results($wpdb->prepare('SELECT id, last_name, first_names, measurement_date, scenario, updated_at FROM %i WHERE created_by = %d ORDER BY COALESCE(measurement_date, DATE(updated_at)) DESC, id DESC LIMIT 2000', $table, get_current_user_id()), ARRAY_A);
        }
        if ($rows === null) { wp_send_json_error(['message' => __('Could not read records.', 'orthogrowth')], 500); }
        wp_send_json_success(['records' => $rows]);
    }

    private static function sanitize_nested_value($value, $depth = 0) {
        if ($depth > 10) { return null; }
        if (is_string($value)) { return sanitize_text_field(wp_check_invalid_utf8($value)); }
        if (is_int($value) || is_bool($value) || $value === null) { return $value; }
        if (is_float($value)) { return is_finite($value) && abs($value) <= 10000000 ? $value : null; }
        if (is_array($value)) {
            if (count($value) > 500) { return null; }
            $clean = [];
            foreach ($value as $key => $item) {
                $clean_key = is_int($key) ? $key : sanitize_key((string) $key);
                if ($clean_key === '' && !is_int($key)) { continue; }
                $clean[$clean_key] = self::sanitize_nested_value($item, $depth + 1);
            }
            return $clean;
        }
        return null;
    }

    public static function ajax_save() {
        self::verify_ajax(); global $wpdb;
        // phpcs:ignore WordPress.Security.NonceVerification.Missing, WordPress.Security.ValidatedSanitizedInput.InputNotSanitized -- verify_ajax() validates the nonce and capability; JSON is decoded before field-by-field validation because sanitizing serialized JSON would corrupt its structure.
        $raw = isset($_POST['record']) && is_string($_POST['record']) ? wp_unslash($_POST['record']) : '';
        if ($raw === '' || strlen($raw) > 2 * 1024 * 1024) { wp_send_json_error(['message' => __('The record is empty or exceeds the 2 MB limit.', 'orthogrowth')], 400); }
        $record = json_decode($raw, true);
        if (!is_array($record) || json_last_error() !== JSON_ERROR_NONE) { wp_send_json_error(['message' => __('Invalid record JSON.', 'orthogrowth')], 400); }

        $patient = isset($record['patient']) && is_array($record['patient']) ? $record['patient'] : [];
        $last_name = isset($patient['last_name']) && is_string($patient['last_name']) ? sanitize_text_field($patient['last_name']) : '';
        $first_names = isset($patient['first_names']) && is_string($patient['first_names']) ? sanitize_text_field($patient['first_names']) : '';
        $last_name = function_exists('mb_substr') ? mb_substr($last_name, 0, 190) : substr($last_name, 0, 190);
        $first_names = function_exists('mb_substr') ? mb_substr($first_names, 0, 190) : substr($first_names, 0, 190);
        if ($last_name === '' || $first_names === '') { wp_send_json_error(['message' => __('Last name and first name(s) are required.', 'orthogrowth')], 400); }
        if (!isset($record['input_values'], $record['result_display']) || !is_array($record['input_values']) || !is_array($record['result_display']) || !$record['input_values'] || !$record['result_display']) {
            wp_send_json_error(['message' => __('Complete a valid calculation before saving.', 'orthogrowth')], 400);
        }
        $scenario = isset($record['scenario']) && is_string($record['scenario']) ? $record['scenario'] : '';
        if (!in_array($scenario, ['congenital', 'developmental'], true)) { wp_send_json_error(['message' => __('Invalid calculation scenario.', 'orthogrowth')], 400); }
        if (isset($record['sex']) && !in_array($record['sex'], ['boys', 'girls'], true)) { wp_send_json_error(['message' => __('Invalid sex value for the multiplier table.', 'orthogrowth')], 400); }
        if (isset($record['unit']) && !in_array($record['unit'], ['mm', 'cm'], true)) { wp_send_json_error(['message' => __('Invalid measurement unit.', 'orthogrowth')], 400); }
        if (isset($record['age_basis']) && !in_array($record['age_basis'], ['chronological', 'bone'], true)) { wp_send_json_error(['message' => __('Invalid age basis.', 'orthogrowth')], 400); }

        $measurement_date = isset($record['measurement_date']) && is_string($record['measurement_date']) ? sanitize_text_field($record['measurement_date']) : '';
        if (!preg_match('/^\d{4}-\d{2}-\d{2}$/', $measurement_date) || !checkdate((int) substr($measurement_date, 5, 2), (int) substr($measurement_date, 8, 2), (int) substr($measurement_date, 0, 4))) { $measurement_date = null; }

        $allowed_top_level = ['application','version','date','sex','chronological_age_years','bone_age_years','bone_age_method','age_basis','age_used_years','measurement_date','previous_measurement_date','previous_measurement_age_years','unit','scenario','measurements','developmental_model','multiplier','multiplier_interpolated','current_discrepancy_signed_right_minus_left','maturity_discrepancy_signed_right_minus_left','maturity_femoral_discrepancy_signed_right_minus_left','maturity_tibial_discrepancy_signed_right_minus_left','estimated_remaining_growth_right','estimated_remaining_growth_left','epiphysiodesis_estimates_age_years','model','patient','input_values','result_display','saved_at'];
        $clean_record = [];
        foreach ($allowed_top_level as $key) {
            if (array_key_exists($key, $record)) { $clean_record[$key] = self::sanitize_nested_value($record[$key]); }
        }
        $allowed_input_ids = ['lastName','firstNames','dob','examDate','age','sex','boneAge','boneMethod','ageBasis','unit','femurR','femurL','tibiaR','tibiaL','footR','footL','prevExamDate','prevAge','prevFemurR','prevFemurL','prevTibiaR','prevTibiaL','scenario'];
        $clean_inputs = [];
        foreach ($allowed_input_ids as $key) {
            if (array_key_exists($key, $record['input_values'])) {
                $value = $record['input_values'][$key];
                if (!is_scalar($value) && $value !== null) { wp_send_json_error(['message' => __('Invalid input value in the saved record.', 'orthogrowth')], 400); }
                $clean_inputs[$key] = is_string($value) ? sanitize_text_field($value) : $value;
            }
        }
        $allowed_result_ids = ['multResult','currentLLD','matureLLD','matureFemur','matureTibia','growth','epiFemurResult','epiTibiaResult','epiBothResult'];
        $clean_results = [];
        foreach ($allowed_result_ids as $key) {
            if (array_key_exists($key, $record['result_display'])) {
                if (!is_scalar($record['result_display'][$key]) && $record['result_display'][$key] !== null) { wp_send_json_error(['message' => __('Invalid calculation result in the saved record.', 'orthogrowth')], 400); }
                $clean_results[$key] = is_string($record['result_display'][$key]) ? sanitize_text_field($record['result_display'][$key]) : $record['result_display'][$key];
            }
        }
        if (!$clean_inputs || !$clean_results) { wp_send_json_error(['message' => __('The record does not contain recognized inputs and results.', 'orthogrowth')], 400); }
        $clean_record['patient'] = ['last_name' => $last_name, 'first_names' => $first_names];
        $clean_record['input_values'] = $clean_inputs;
        $clean_record['result_display'] = $clean_results;
        $clean_record['scenario'] = $scenario;
        $clean_record['measurement_date'] = $measurement_date;
        $clean_record['application'] = 'OrthoGrowth WordPress';
        $clean_record['version'] = self::VERSION;
        $json = wp_json_encode($clean_record, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
        if (!$json) { wp_send_json_error(['message' => __('Could not prepare the record for storage.', 'orthogrowth')], 500); }
        $now = current_time('mysql', true); $user_id = get_current_user_id();
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Inserts a validated record into OrthoGrowth's custom table; no object-cache entry is created for records.
        $ok = $wpdb->insert(self::table_name(), ['last_name'=>$last_name,'first_names'=>$first_names,'measurement_date'=>$measurement_date,'scenario'=>$scenario,'record_json'=>$json,'created_by'=>$user_id,'updated_by'=>$user_id,'created_at'=>$now,'updated_at'=>$now], ['%s','%s','%s','%s','%s','%d','%d','%s','%s']);
        if (!$ok) { wp_send_json_error(['message' => __('Database storage failed.', 'orthogrowth')], 500); }
        wp_send_json_success(['id' => (int)$wpdb->insert_id]);
    }

    private static function get_authorized_record($id) {
        global $wpdb; $table = self::table_name();
        if (self::is_admin_user()) {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Fetches one authorized row from the custom table; sensitive records are deliberately not cached.
            return $wpdb->get_row($wpdb->prepare('SELECT id, record_json FROM %i WHERE id = %d', $table, $id), ARRAY_A);
        }
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Fetches one user-scoped row from the custom table; sensitive records are deliberately not cached.
        return $wpdb->get_row($wpdb->prepare('SELECT id, record_json FROM %i WHERE id = %d AND created_by = %d', $table, $id, get_current_user_id()), ARRAY_A);
    }

    public static function ajax_get() {
        self::verify_ajax();
        // phpcs:ignore WordPress.Security.NonceVerification.Missing -- verify_ajax() validates the nonce and capability before reading this request value.
        $id = isset($_POST['id']) ? absint($_POST['id']) : 0;
        if (!$id) { wp_send_json_error(['message' => __('Invalid record ID.', 'orthogrowth')], 400); }
        $row = self::get_authorized_record($id);
        if (!$row) { wp_send_json_error(['message' => __('Record not found or access denied.', 'orthogrowth')], 404); }
        $record = json_decode($row['record_json'], true);
        if (!is_array($record)) { wp_send_json_error(['message' => __('The saved record cannot be read.', 'orthogrowth')], 500); }
        $record['id'] = $id; wp_send_json_success(['record' => $record]);
    }

    public static function ajax_delete() {
        self::verify_ajax(); global $wpdb;
        // phpcs:ignore WordPress.Security.NonceVerification.Missing -- verify_ajax() validates the nonce and capability before reading this request value.
        $id = isset($_POST['id']) ? absint($_POST['id']) : 0;
        if (!$id) { wp_send_json_error(['message' => __('Invalid record ID.', 'orthogrowth')], 400); }
        $table = self::table_name();
        if (self::is_admin_user()) {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Deletes one selected record from the custom table; no cache is maintained.
            $deleted = $wpdb->delete($table, ['id'=>$id], ['%d']);
        } else {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Deletes only a record owned by the current user; no cache is maintained.
            $deleted = $wpdb->delete($table, ['id'=>$id, 'created_by'=>get_current_user_id()], ['%d','%d']);
        }
        if ($deleted === false) { wp_send_json_error(['message' => __('Deletion failed.', 'orthogrowth')], 500); }
        if ($deleted === 0) { wp_send_json_error(['message' => __('Record not found or deletion not permitted.', 'orthogrowth')], 404); }
        wp_send_json_success(['deleted' => true]);
    }
}

OrthoGrowth::init();
