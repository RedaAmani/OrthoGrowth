<?php
if (!defined('WP_UNINSTALL_PLUGIN')) { exit; }

function orthogrowth_uninstall_current_site() {
    global $wpdb;
    if ((bool) get_option('orthogrowth_delete_data_on_uninstall', 0)) {
        $table = $wpdb->prefix . 'orthogrowth_records';
        $wpdb->query($wpdb->prepare('DROP TABLE IF EXISTS %i', $table));
    }
    foreach (get_users(['fields' => 'ID', 'number' => 0]) as $user_id) {
        $user = new WP_User((int) $user_id);
        $user->remove_cap('orthogrowth_use');
    }
    $role = get_role('administrator');
    if ($role) { $role->remove_cap('orthogrowth_use'); }
    delete_option('orthogrowth_db_version');
    delete_option('orthogrowth_delete_data_on_uninstall');
}

if (is_multisite() && get_site_option('orthogrowth_network_active', 0)) {
    $site_ids = get_sites(['number' => 0, 'fields' => 'ids']);
    foreach ($site_ids as $site_id) {
        switch_to_blog((int) $site_id);
        orthogrowth_uninstall_current_site();
        restore_current_blog();
    }
    delete_site_option('orthogrowth_network_active');
} else {
    orthogrowth_uninstall_current_site();
}
