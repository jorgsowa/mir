===description===
`array_key_exists('b', $arr['a'])` proves `$arr['a']['b']` present, the same
way `isset($arr['a']['b'])` narrowing already does for a nested path —
previously only a plain-variable or single-level-property array argument was
handled.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array{a: array{b?: string}} $arr */
function f(array $arr): void {
    if (array_key_exists('b', $arr['a'])) {
        takesString($arr['a']['b']);
    }
}
function takesString(string $s): void {}
===expect===
