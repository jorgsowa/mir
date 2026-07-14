===description===
key_exists() is a built-in alias of array_key_exists() and must get the
same shape-narrowing: a guarded access is fine, an unguarded one still
flags a nonexistent key.
===file===
<?php
/** @param array{title: string} $meta */
function guarded(array $meta): string {
    return key_exists('favicon', $meta) ? (string) $meta['favicon'] : '';
}
/** @param array{title: string} $meta */
function unguarded(array $meta): string {
    return (string) $meta['favicon'];
}
===expect===
NonExistentArrayOffset@8:26-8:35: Array offset 'favicon' does not exist
