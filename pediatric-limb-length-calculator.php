<?php
/**
 * Plugin Name: Pediatric Limb Length Calculator
 * Description: Browser-based estimates of limb-length discrepancy and theoretical epiphysiodesis timing.
 * Plugin URI: https://github.com/RedaAmani/OrthoGrowth
 * Version: 0.6.2
 * Requires at least: 6.2
 * Requires PHP: 8.0
 * Author: Dr M.R. Amani
 * License: GPL-2.0-or-later
 * License URI: https://www.gnu.org/licenses/gpl-2.0.html
 * Text Domain: pediatric-limb-length-calculator
 */

if (!defined('ABSPATH')) { exit; }

final class Pediatric_Limb_Length_Calculator {
    const VERSION = '0.6.2';
    const DB_VERSION = '1.0';
    const CAPABILITY = 'pediatric_limb_length_calculator_use';
    const NONCE_ACTION = 'pediatric_limb_length_calculator_server_action';
    const DELETE_OPTION = 'pediatric_limb_length_calculator_delete_data_on_uninstall';

    public static function init() {
        register_activation_hook(__FILE__, [__CLASS__, 'activate']);
        add_action('plugins_loaded', [__CLASS__, 'maybe_upgrade_schema']);
        add_action('wp_initialize_site', [__CLASS__, 'initialize_new_site'], 10, 1);
        add_shortcode('pediatric-limb-length-calculator', [__CLASS__, 'shortcode']);
        add_action('wp_enqueue_scripts', [__CLASS__, 'enqueue_frontend_assets']);
        add_action('admin_menu', [__CLASS__, 'admin_menu']);
        add_action('admin_post_pediatric_limb_length_calculator_access', [__CLASS__, 'save_access']);
        add_action('admin_post_pediatric_limb_length_calculator_settings', [__CLASS__, 'save_settings']);
        add_action('wp_ajax_pediatric_limb_length_calculator_list', [__CLASS__, 'ajax_list']);
        add_action('wp_ajax_pediatric_limb_length_calculator_save', [__CLASS__, 'ajax_save']);
        add_action('wp_ajax_pediatric_limb_length_calculator_get', [__CLASS__, 'ajax_get']);
        add_action('wp_ajax_pediatric_limb_length_calculator_delete', [__CLASS__, 'ajax_delete']);
        add_action('admin_init', [__CLASS__, 'register_privacy_policy_content']);
        add_filter('wp_privacy_personal_data_exporters', [__CLASS__, 'register_privacy_exporter']);
        add_filter('wp_privacy_personal_data_erasers', [__CLASS__, 'register_privacy_eraser']);
    }

    public static function table_name() {
        global $wpdb;
        return $wpdb->prefix . 'pediatric_limb_length_calculator_records';
    }

    public static function activate($network_wide = false) {
        if (is_multisite() && $network_wide) {
            update_site_option('pediatric_limb_length_calculator_network_active', 1);
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
        if (!is_multisite() || !get_site_option('pediatric_limb_length_calculator_network_active', 0)) { return; }
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
        update_option('pediatric_limb_length_calculator_db_version', self::DB_VERSION, false);
        if (get_option(self::DELETE_OPTION, null) === null) { add_option(self::DELETE_OPTION, 0, '', false); }
    }

    public static function maybe_upgrade_schema() {
        if (get_option('pediatric_limb_length_calculator_db_version') !== self::DB_VERSION) {
            self::install_site();
        }
    }

    private static function can_use() {
        return is_user_logged_in() && current_user_can(self::CAPABILITY);
    }

    private static function is_admin_user() { return current_user_can('manage_options'); }

    public static function enqueue_frontend_assets() {
        global $post;
        if (!($post instanceof WP_Post) || !has_shortcode($post->post_content, 'pediatric-limb-length-calculator')) { return; }
        self::enqueue_assets();
    }

    private static function enqueue_assets() {
        wp_enqueue_style('pediatric_limb_length_calculator-app', plugins_url('pediatric-limb-length-calculator.css', __FILE__), [], self::VERSION);
        wp_enqueue_script('pediatric_limb_length_calculator-app', plugins_url('pediatric-limb-length-calculator.js', __FILE__), [], self::VERSION, true);
        $locale = determine_locale();
        $initial_language = (strpos(strtolower($locale), 'fr') === 0) ? 'fr' : 'en';
        wp_localize_script('pediatric_limb_length_calculator-app', 'PediatricLimbLengthCalculatorConfig', [
            'ajaxUrl' => admin_url('admin-ajax.php'),
            'nonce' => wp_create_nonce(self::NONCE_ACTION),
            'initialLanguage' => $initial_language,
        ]);
    }

    public static function shortcode() {
        if (!is_user_logged_in()) {
            $url = wp_login_url(get_permalink());
            return '<div class="pediatric_limb_length_calculator-access-message"><p>' . esc_html__('This application is restricted to authorized users.', 'pediatric-limb-length-calculator') . '</p><p><a href="' . esc_url($url) . '">' . esc_html__('Log in', 'pediatric-limb-length-calculator') . '</a></p></div>';
        }
        if (!self::can_use()) {
            return '<div class="pediatric_limb_length_calculator-access-message"><p>' . esc_html__('Your account is not authorized to use Pediatric Limb Length Calculator. Contact the site administrator.', 'pediatric-limb-length-calculator') . '</p></div>';
        }
        $path = plugin_dir_path(__FILE__) . 'app-template.html';
        if (!is_readable($path)) { return '<p>' . esc_html__('The Pediatric Limb Length Calculator interface template is missing. Reinstall the plugin.', 'pediatric-limb-length-calculator') . '</p>'; }
        $template = file_get_contents($path);
        if ($template === false) { return '<p>' . esc_html__('The Pediatric Limb Length Calculator interface could not be loaded.', 'pediatric-limb-length-calculator') . '</p>'; }
        self::enqueue_assets();
        // The template is plugin-owned markup, but sanitize its HTML before returning it through the shortcode.
        return wp_kses($template, self::allowed_template_html());
    }

    /**
     * HTML elements and attributes required by the static application template.
     * Keep this allowlist narrow: the template does not need scripts or event-handler attributes.
     */
    private static function allowed_template_html() {
        return [
            'div' => ['id' => true, 'class' => true, 'style' => true, 'role' => true],
            'header' => ['id' => true, 'class' => true],
            'main' => ['id' => true, 'class' => true],
            'section' => ['id' => true, 'class' => true],
            'h1' => ['id' => true, 'class' => true, 'style' => true],
            'h2' => ['id' => true, 'class' => true, 'style' => true],
            'p' => ['id' => true, 'class' => true, 'style' => true],
            'label' => ['id' => true, 'class' => true, 'for' => true],
            'input' => ['id' => true, 'class' => true, 'type' => true, 'autocomplete' => true, 'maxlength' => true, 'min' => true, 'max' => true, 'step' => true, 'placeholder' => true, 'aria-label' => true],
            'select' => ['id' => true, 'class' => true],
            'option' => ['value' => true],
            'table' => ['id' => true, 'class' => true],
            'thead' => ['id' => true, 'class' => true],
            'tbody' => ['id' => true, 'class' => true],
            'tr' => ['id' => true, 'class' => true],
            'th' => ['id' => true, 'class' => true],
            'td' => ['id' => true, 'class' => true],
            'button' => ['id' => true, 'class' => true, 'type' => true],
            'small' => ['id' => true, 'class' => true],
            'details' => ['id' => true, 'class' => true, 'style' => true],
            'summary' => ['id' => true, 'class' => true],
            'a' => ['id' => true, 'class' => true, 'href' => true, 'target' => true, 'rel' => true],
        ];
    }

    public static function admin_menu() {
        add_menu_page(__('Pediatric Limb Length Calculator', 'pediatric-limb-length-calculator'), __('Pediatric Limb Length Calculator', 'pediatric-limb-length-calculator'), 'manage_options', 'pediatric_limb_length_calculator-access', [__CLASS__, 'access_page'], 'dashicons-analytics', 58);
    }

    public static function access_page() {
        if (!current_user_can('manage_options')) { return; }
        $users = get_users(['orderby' => 'display_name', 'order' => 'ASC']);
        echo '<div class="wrap"><h1>' . esc_html__('Pediatric Limb Length Calculator access', 'pediatric-limb-length-calculator') . '</h1>';
        echo '<p>' . esc_html__('Authorized users can view and delete only the records they created. Administrators can view and delete all records.', 'pediatric-limb-length-calculator') . '</p>';
        // phpcs:ignore WordPress.Security.NonceVerification.Recommended -- Read-only status flag for the redirect notice; it does not trigger a state change.
        if (isset($_GET['updated']) && '1' === sanitize_text_field(wp_unslash($_GET['updated']))) { echo '<div class="notice notice-success is-dismissible"><p>' . esc_html__('Settings saved.', 'pediatric-limb-length-calculator') . '</p></div>'; }
        echo '<form method="post" action="' . esc_url(admin_url('admin-post.php')) . '"><input type="hidden" name="action" value="pediatric_limb_length_calculator_access">';
        wp_nonce_field('pediatric_limb_length_calculator_access_save');
        echo '<table class="widefat striped"><thead><tr><th>' . esc_html__('Authorized', 'pediatric-limb-length-calculator') . '</th><th>' . esc_html__('Display name', 'pediatric-limb-length-calculator') . '</th><th>' . esc_html__('Username', 'pediatric-limb-length-calculator') . '</th><th>' . esc_html__('Email', 'pediatric-limb-length-calculator') . '</th><th>' . esc_html__('Role', 'pediatric-limb-length-calculator') . '</th></tr></thead><tbody>';
        foreach ($users as $user) {
            $is_admin = user_can($user, 'manage_options');
            $checked = $is_admin || user_can($user, self::CAPABILITY);
            echo '<tr><td><input type="checkbox" name="allowed_users[]" value="' . esc_attr($user->ID) . '" ' . checked($checked, true, false) . ($is_admin ? ' disabled' : '') . '></td>';
            echo '<td>' . esc_html($user->display_name) . '</td><td>' . esc_html($user->user_login) . '</td><td>' . esc_html($user->user_email) . '</td><td>' . esc_html(implode(', ', $user->roles)) . ($is_admin ? ' (' . esc_html__('administrator, access retained', 'pediatric-limb-length-calculator') . ')' : '') . '</td></tr>';
        }
        echo '</tbody></table><p><button class="button button-primary" type="submit">' . esc_html__('Save access permissions', 'pediatric-limb-length-calculator') . '</button></p></form>';
        echo '<hr><h2>' . esc_html__('Data deletion on uninstall', 'pediatric-limb-length-calculator') . '</h2>';
        echo '<p>' . esc_html__('By default, uninstalling Pediatric Limb Length Calculator preserves saved records. You may explicitly choose to permanently delete all Pediatric Limb Length Calculator records when the plugin is uninstalled. This deletion is irreversible.', 'pediatric-limb-length-calculator') . '</p>';
        echo '<form method="post" action="' . esc_url(admin_url('admin-post.php')) . '"><input type="hidden" name="action" value="pediatric_limb_length_calculator_settings">';
        wp_nonce_field('pediatric_limb_length_calculator_settings_save');
        echo '<label><input type="checkbox" name="delete_data_on_uninstall" value="1" ' . checked((bool)get_option(self::DELETE_OPTION, 0), true, false) . '> ' . esc_html__('Permanently delete all saved records when Pediatric Limb Length Calculator is uninstalled', 'pediatric-limb-length-calculator') . '</label><p><button class="button button-primary" type="submit">' . esc_html__('Save data deletion setting', 'pediatric-limb-length-calculator') . '</button></p></form></div>';
    }

    public static function save_access() {
        if (!current_user_can('manage_options')) { wp_die(esc_html__('Access denied.', 'pediatric-limb-length-calculator'), 403); }
        check_admin_referer('pediatric_limb_length_calculator_access_save');
        $allowed = isset($_POST['allowed_users']) ? array_map('absint', (array)wp_unslash($_POST['allowed_users'])) : [];
        $admins = array_map('intval', get_users(['role' => 'administrator', 'fields' => 'ID']));
        foreach (get_users() as $user) {
            $id = (int)$user->ID;
            if (in_array($id, $admins, true)) { continue; }
            $wp_user = new WP_User($id);
            if (in_array($id, $allowed, true)) { $wp_user->add_cap(self::CAPABILITY); }
            else { $wp_user->remove_cap(self::CAPABILITY); }
        }
        wp_safe_redirect(add_query_arg(['page' => 'pediatric_limb_length_calculator-access', 'updated' => '1'], admin_url('admin.php'))); exit;
    }

    public static function save_settings() {
        if (!current_user_can('manage_options')) { wp_die(esc_html__('Access denied.', 'pediatric-limb-length-calculator'), 403); }
        check_admin_referer('pediatric_limb_length_calculator_settings_save');
        update_option(self::DELETE_OPTION, isset($_POST['delete_data_on_uninstall']) ? 1 : 0, false);
        wp_safe_redirect(add_query_arg(['page' => 'pediatric_limb_length_calculator-access', 'updated' => '1'], admin_url('admin.php'))); exit;
    }

    public static function register_privacy_policy_content() {
        if (function_exists('wp_add_privacy_policy_content')) {
            $content = '<p>' . esc_html__('Pediatric Limb Length Calculator stores patient names, measurement dates, entered measurements and calculation results in the WordPress database when a user saves a record. Records are accessible to the user who created them and to site administrators. The WordPress personal-data export and erasure tools can export or delete records associated with a user account. The site owner is responsible for setting an appropriate retention period, protecting backups and complying with applicable health-data and privacy laws.', 'pediatric-limb-length-calculator') . '</p>';
            wp_add_privacy_policy_content('Pediatric Limb Length Calculator', $content);
        }
    }

    public static function register_privacy_exporter($exporters) {
        $exporters['pediatric-limb-length-calculator'] = [
            'exporter_friendly_name' => __('Pediatric Limb Length Calculator records', 'pediatric-limb-length-calculator'),
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
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- This paginated query reads Pediatric Limb Length Calculator's custom table; sensitive records are deliberately not cached.
        $rows = $wpdb->get_results($wpdb->prepare(
            'SELECT id, last_name, first_names, measurement_date, scenario, record_json, created_at, updated_at FROM %i WHERE created_by = %d ORDER BY id ASC LIMIT %d OFFSET %d',
            self::table_name(), (int) $user->ID, $limit, $offset
        ), ARRAY_A);
        if (!is_array($rows)) { return ['data' => [], 'done' => true]; }
        $items = [];
        foreach ($rows as $row) {
            $items[] = [
                'group_id' => 'pediatric_limb_length_calculator-records',
                'group_label' => __('Pediatric Limb Length Calculator records', 'pediatric-limb-length-calculator'),
                'item_id' => 'pediatric_limb_length_calculator-record-' . (int) $row['id'],
                'data' => [
                    ['name' => __('Patient last name', 'pediatric-limb-length-calculator'), 'value' => $row['last_name']],
                    ['name' => __('Patient first name(s)', 'pediatric-limb-length-calculator'), 'value' => $row['first_names']],
                    ['name' => __('Measurement date', 'pediatric-limb-length-calculator'), 'value' => $row['measurement_date'] ?? ''],
                    ['name' => __('Scenario', 'pediatric-limb-length-calculator'), 'value' => $row['scenario']],
                    ['name' => __('Record data (JSON)', 'pediatric-limb-length-calculator'), 'value' => $row['record_json']],
                    ['name' => __('Record created (UTC)', 'pediatric-limb-length-calculator'), 'value' => $row['created_at']],
                    ['name' => __('Record updated (UTC)', 'pediatric-limb-length-calculator'), 'value' => $row['updated_at']],
                ],
            ];
        }
        return ['data' => $items, 'done' => count($rows) < $limit];
    }

    public static function register_privacy_eraser($erasers) {
        $erasers['pediatric-limb-length-calculator'] = [
            'eraser_friendly_name' => __('Pediatric Limb Length Calculator records', 'pediatric-limb-length-calculator'),
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
            return ['items_removed' => false, 'items_retained' => true, 'messages' => [__('Pediatric Limb Length Calculator records could not be deleted. The site administrator should investigate the database error.', 'pediatric-limb-length-calculator')], 'done' => true];
        }
        return ['items_removed' => $deleted > 0, 'items_retained' => false, 'messages' => [], 'done' => true];
    }

    private static function verify_ajax() {
        if (!is_user_logged_in()) { wp_send_json_error(['message' => __('Session expired. Please log in again.', 'pediatric-limb-length-calculator')], 401); }
        if (!self::can_use()) { wp_send_json_error(['message' => __('Access denied.', 'pediatric-limb-length-calculator')], 403); }
        // Use WordPress' AJAX nonce verifier so the check is explicit and recognized by security tooling.
        if (false === check_ajax_referer(self::NONCE_ACTION, 'nonce', false)) {
            wp_send_json_error(['message' => __('Security token expired. Reload the page.', 'pediatric-limb-length-calculator')], 403);
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
        if ($rows === null) { wp_send_json_error(['message' => __('Could not read records.', 'pediatric-limb-length-calculator')], 500); }
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
        if ($raw === '' || strlen($raw) > 2 * 1024 * 1024) { wp_send_json_error(['message' => __('The record is empty or exceeds the 2 MB limit.', 'pediatric-limb-length-calculator')], 400); }
        $record = json_decode($raw, true);
        if (!is_array($record) || json_last_error() !== JSON_ERROR_NONE) { wp_send_json_error(['message' => __('Invalid record JSON.', 'pediatric-limb-length-calculator')], 400); }

        $patient = isset($record['patient']) && is_array($record['patient']) ? $record['patient'] : [];
        $last_name = isset($patient['last_name']) && is_string($patient['last_name']) ? sanitize_text_field($patient['last_name']) : '';
        $first_names = isset($patient['first_names']) && is_string($patient['first_names']) ? sanitize_text_field($patient['first_names']) : '';
        $last_name = function_exists('mb_substr') ? mb_substr($last_name, 0, 190) : substr($last_name, 0, 190);
        $first_names = function_exists('mb_substr') ? mb_substr($first_names, 0, 190) : substr($first_names, 0, 190);
        if ($last_name === '' || $first_names === '') { wp_send_json_error(['message' => __('Last name and first name(s) are required.', 'pediatric-limb-length-calculator')], 400); }
        if (!isset($record['input_values'], $record['result_display']) || !is_array($record['input_values']) || !is_array($record['result_display']) || !$record['input_values'] || !$record['result_display']) {
            wp_send_json_error(['message' => __('Complete a valid calculation before saving.', 'pediatric-limb-length-calculator')], 400);
        }
        $scenario = isset($record['scenario']) && is_string($record['scenario']) ? $record['scenario'] : '';
        if (!in_array($scenario, ['congenital', 'developmental'], true)) { wp_send_json_error(['message' => __('Invalid calculation scenario.', 'pediatric-limb-length-calculator')], 400); }
        if (isset($record['sex']) && !in_array($record['sex'], ['boys', 'girls'], true)) { wp_send_json_error(['message' => __('Invalid sex value for the multiplier table.', 'pediatric-limb-length-calculator')], 400); }
        if (isset($record['unit']) && !in_array($record['unit'], ['mm', 'cm'], true)) { wp_send_json_error(['message' => __('Invalid measurement unit.', 'pediatric-limb-length-calculator')], 400); }
        if (isset($record['age_basis']) && !in_array($record['age_basis'], ['chronological', 'bone'], true)) { wp_send_json_error(['message' => __('Invalid age basis.', 'pediatric-limb-length-calculator')], 400); }

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
                if (!is_scalar($value) && $value !== null) { wp_send_json_error(['message' => __('Invalid input value in the saved record.', 'pediatric-limb-length-calculator')], 400); }
                $clean_inputs[$key] = is_string($value) ? sanitize_text_field($value) : $value;
            }
        }
        $allowed_result_ids = ['multResult','currentLLD','matureLLD','matureFemur','matureTibia','growth','epiFemurResult','epiTibiaResult','epiBothResult'];
        $clean_results = [];
        foreach ($allowed_result_ids as $key) {
            if (array_key_exists($key, $record['result_display'])) {
                if (!is_scalar($record['result_display'][$key]) && $record['result_display'][$key] !== null) { wp_send_json_error(['message' => __('Invalid calculation result in the saved record.', 'pediatric-limb-length-calculator')], 400); }
                $clean_results[$key] = is_string($record['result_display'][$key]) ? sanitize_text_field($record['result_display'][$key]) : $record['result_display'][$key];
            }
        }
        if (!$clean_inputs || !$clean_results) { wp_send_json_error(['message' => __('The record does not contain recognized inputs and results.', 'pediatric-limb-length-calculator')], 400); }
        $clean_record['patient'] = ['last_name' => $last_name, 'first_names' => $first_names];
        $clean_record['input_values'] = $clean_inputs;
        $clean_record['result_display'] = $clean_results;
        $clean_record['scenario'] = $scenario;
        $clean_record['measurement_date'] = $measurement_date;
        $clean_record['application'] = 'Pediatric Limb Length Calculator WordPress';
        $clean_record['version'] = self::VERSION;
        $json = wp_json_encode($clean_record, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
        if (!$json) { wp_send_json_error(['message' => __('Could not prepare the record for storage.', 'pediatric-limb-length-calculator')], 500); }
        $now = current_time('mysql', true); $user_id = get_current_user_id();
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Inserts a validated record into Pediatric Limb Length Calculator's custom table; no object-cache entry is created for records.
        $ok = $wpdb->insert(self::table_name(), ['last_name'=>$last_name,'first_names'=>$first_names,'measurement_date'=>$measurement_date,'scenario'=>$scenario,'record_json'=>$json,'created_by'=>$user_id,'updated_by'=>$user_id,'created_at'=>$now,'updated_at'=>$now], ['%s','%s','%s','%s','%s','%d','%d','%s','%s']);
        if (!$ok) { wp_send_json_error(['message' => __('Database storage failed.', 'pediatric-limb-length-calculator')], 500); }
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
        if (!$id) { wp_send_json_error(['message' => __('Invalid record ID.', 'pediatric-limb-length-calculator')], 400); }
        $row = self::get_authorized_record($id);
        if (!$row) { wp_send_json_error(['message' => __('Record not found or access denied.', 'pediatric-limb-length-calculator')], 404); }
        $record = json_decode($row['record_json'], true);
        if (!is_array($record)) { wp_send_json_error(['message' => __('The saved record cannot be read.', 'pediatric-limb-length-calculator')], 500); }
        $record['id'] = $id; wp_send_json_success(['record' => $record]);
    }

    public static function ajax_delete() {
        self::verify_ajax(); global $wpdb;
        // phpcs:ignore WordPress.Security.NonceVerification.Missing -- verify_ajax() validates the nonce and capability before reading this request value.
        $id = isset($_POST['id']) ? absint($_POST['id']) : 0;
        if (!$id) { wp_send_json_error(['message' => __('Invalid record ID.', 'pediatric-limb-length-calculator')], 400); }
        $table = self::table_name();
        if (self::is_admin_user()) {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Deletes one selected record from the custom table; no cache is maintained.
            $deleted = $wpdb->delete($table, ['id'=>$id], ['%d']);
        } else {
            // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching -- Deletes only a record owned by the current user; no cache is maintained.
            $deleted = $wpdb->delete($table, ['id'=>$id, 'created_by'=>get_current_user_id()], ['%d','%d']);
        }
        if ($deleted === false) { wp_send_json_error(['message' => __('Deletion failed.', 'pediatric-limb-length-calculator')], 500); }
        if ($deleted === 0) { wp_send_json_error(['message' => __('Record not found or deletion not permitted.', 'pediatric-limb-length-calculator')], 404); }
        wp_send_json_success(['deleted' => true]);
    }
}

Pediatric_Limb_Length_Calculator::init();
