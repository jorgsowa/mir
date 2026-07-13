===description===
Sibling of keyed_destructure_optional_key_includes_null: a required
(non-optional) shape key stays exactly its declared type, no null.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{a: string} $arr
 */
function test(array $arr): void {
    ['a' => $a] = $arr;
    /** @trace $a */
    strlen($a);
}
===expect===
Trace@8:4-8:15: Type of $a is string
