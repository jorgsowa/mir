===description===
`!array_key_exists('x', $arr['a'])`'s false branch also proves `$arr['a']`
is a real array (evaluating array_key_exists's second argument required
it) — the outer key `a` must no longer be optional there either, mirroring
the already-correct true-branch clearing.
===config===
suppress=UnusedParam,UnusedVariable,PossiblyNullArgument
===file===
<?php
/** @param array{a?: array{x?: int}} $arr */
function f(array $arr): void {
    if (!array_key_exists('x', $arr['a'])) {
        /** @mir-check $arr is array{'a': array{'x'?: int}} */
        $val = $arr['a'];
        /** @mir-check $val is array{'x'?: int} */
        echo 1;
    }
}
===expect===
