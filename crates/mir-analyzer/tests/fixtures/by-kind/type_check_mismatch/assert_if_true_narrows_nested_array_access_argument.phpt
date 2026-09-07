===description===
`@psalm-assert-if-true string $arr['a']['b']` targets a nested key path of
the `$arr` parameter — `split_array_key_suffix` used to bail on a second
`[` entirely, leaving the target name untouched (`"arr['a']['b']"`), which
could never match a real declared parameter name, so the whole assertion
silently no-oped for a nested path even though the single-key case worked.
===config===
suppress=MissingReturnType,MixedArgument
===file===
<?php
/**
 * @param array{a?: array{b?: string}} $config
 * @psalm-assert-if-true string $config['a']['b']
 */
function hasNestedStatus(array $config) {
    return isset($config['a']['b']);
}

/** @param array{a?: array{b?: string}} $c */
function test(array $c): void {
    if (hasNestedStatus($c)) {
        /** @mir-check $c is array{a: array{b: string}} */
        $_ = 1;
    }
}
===expect===
