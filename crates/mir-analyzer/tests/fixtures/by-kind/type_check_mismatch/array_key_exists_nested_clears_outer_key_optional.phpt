===description===
`array_key_exists('x', $arr['a'])` proves `$arr['a']` is a real array, so the
outer key `a` is no longer optional — even when the inner key `x` is already
declared present (so the recursive narrowing into `a`'s own value returns no
change on its own, which previously meant the outer key's optionality was
never cleared either).
===config===
suppress=UnusedParam,UnusedVariable,PossiblyNullArgument
===file===
<?php
/** @param array{a?: array{x: int}} $arr */
function f(array $arr): void {
    if (array_key_exists('x', $arr['a'])) {
        $val = $arr['a'];
        /** @mir-check $val is array{x: int} */
        echo 1;
    }
}
===expect===
