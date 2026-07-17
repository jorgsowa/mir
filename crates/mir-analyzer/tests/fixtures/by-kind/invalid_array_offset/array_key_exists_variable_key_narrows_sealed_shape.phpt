===description===
array_key_exists($key, $arr) resolves $key when it's a variable already
narrowed to a single literal, same as passing the literal inline.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array{title: string} $meta */
function guarded_by_variable_string_key(array $meta): string {
    $key = 'favicon';
    return array_key_exists($key, $meta) ? (string) $meta['favicon'] : '';
}

/** @param array{title: string} $meta */
function guarded_by_variable_key_via_key_exists_alias(array $meta): string {
    $key = 'favicon';
    return key_exists($key, $meta) ? (string) $meta['favicon'] : '';
}

/** @param array{0: string} $list */
function guarded_by_variable_int_key(array $list): string {
    $key = 0;
    return array_key_exists($key, $list) ? $list[0] : '';
}

/** @param array{title: string} $meta */
function not_narrowed_when_key_is_not_a_literal(array $meta, string $key): string {
    // $key is an unnarrowed string, not a proven literal — must not be
    // treated as if it were the literal 'favicon'.
    return array_key_exists($key, $meta) ? (string) $meta['favicon'] : '';
}
===expect===
NonExistentArrayOffset@24:58-24:67: Array offset 'favicon' does not exist
