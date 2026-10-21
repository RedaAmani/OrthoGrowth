<?php
if (!defined('WP_UNINSTALL_PLUGIN')) { exit; }

function pediatric_limb_length_calculator_uninstall_current_site() {
    global $wpdb;
    if ((bool) get_option('pediatric_limb_length_calculator_delete_data_on_uninstall', 0)) {
        $table = $wpdb->prefix . 'pediatric_limb_length_calculator_records';
        // phpcs:ignore WordPress.DB.DirectDatabaseQuery.DirectQuery, WordPress.DB.DirectDatabaseQuery.NoCaching, WordPress.DB.DirectDatabaseQuery.SchemaChange -- Intentional opt-in cleanup of this plugin's custom table during uninstall; no runtime cache exists.
        $wpdb->query($wpdb->prepare('DROP TABLE IF EXISTS %i', $table));
    }
    foreach (get_users(['fields' => 'ID', 'number' => 0]) as $user_id) {
        $user = new WP_User((int) $user_id);
        $user->remove_cap('pediatric_limb_length_calculator_use');
    }
    $role = get_role('administrator');
    if ($role) { $role->remove_cap('pediatric_limb_length_calculator_use'); }
    delete_option('pediatric_limb_length_calculator_db_version');
    delete_option('pediatric_limb_length_calculator_delete_data_on_uninstall');
}

if (is_multisite() && get_site_option('pediatric_limb_length_calculator_network_active', 0)) {
    $pediatric_limb_length_calculator_site_ids = get_sites(['number' => 0, 'fields' => 'ids']);
    foreach ($pediatric_limb_length_calculator_site_ids as $pediatric_limb_length_calculator_site_id) {
        switch_to_blog((int) $pediatric_limb_length_calculator_site_id);
        pediatric_limb_length_calculator_uninstall_current_site();
        restore_current_blog();
    }
    delete_site_option('pediatric_limb_length_calculator_network_active');
} else {
    pediatric_limb_length_calculator_uninstall_current_site();
}
