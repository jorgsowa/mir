===description===
array_change_key_case only case-folds STRING keys; int keys and all values
pass through unchanged. For a plain array<K,V> (not a shape), folding a
string key's TYPE is a no-op — the source type is returned unchanged. For
a keyed-array shape, each string key is rewritten to CASE_LOWER (default)
or CASE_UPPER when the $case argument resolves to a known literal.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int> $assoc */
function test(array $assoc): void {
    $unchanged = array_change_key_case($assoc, CASE_UPPER);
    /** @mir-check $unchanged is array<string, int> */
    $_ = $unchanged;

    $shape = ['Foo' => 1, 'Bar' => 'x'];
    $lower = array_change_key_case($shape);
    /** @mir-check $lower is array{foo: 1, bar: 'x'} */
    $_ = $lower;

    $upper = array_change_key_case($shape, CASE_UPPER);
    /** @mir-check $upper is array{FOO: 1, BAR: 'x'} */
    $_ = $upper;
}
===expect===
