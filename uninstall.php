<?php
if (!defined('WP_UNINSTALL_PLUGIN')) { exit; }
$delete_data = (bool) get_option('orthogrowth_delete_data_on_uninstall', 0);
if ($delete_data) {
    global $wpdb;
    $table = $wpdb->prefix . 'orthogrowth_records';
    $wpdb->query($wpdb->prepare('DROP TABLE IF EXISTS %i', $table));
}
foreach (get_users() as $user) {
    $wp_user = new WP_User($user->ID);
    $wp_user->remove_cap('orthogrowth_use');
}
$role = get_role('administrator');
if ($role) {
    $role->remove_cap('orthogrowth_use');
}
delete_option('orthogrowth_db_version');
delete_option('orthogrowth_delete_data_on_uninstall');
