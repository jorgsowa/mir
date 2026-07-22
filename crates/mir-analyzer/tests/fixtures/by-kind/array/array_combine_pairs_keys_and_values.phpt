===description===
array_combine pairs $keys's values (coerced to a legal array-key type) with
$values's values, positionally. Since PHP 8 throws a ValueError on a count
mismatch, a non-empty $keys guarantees a non-empty result on the successful-
return path. The result is never a list — keys come from $keys's arbitrary
values, not sequential indices.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<string> $names
 * @param non-empty-list<int> $ages
 * @param list<string> $maybe_empty_names
 * @param list<int> $maybe_empty_values
 */
function test(
    array $names,
    array $ages,
    array $maybe_empty_names,
    array $maybe_empty_values
): void {
    $combined = array_combine($names, $ages);
    /** @mir-check $combined is non-empty-array<string, int> */
    $_ = $combined;

    $maybe_empty = array_combine($maybe_empty_names, $maybe_empty_values);
    /** @mir-check $maybe_empty is array<string, int> */
    $_ = $maybe_empty;
}
===expect===
MissingThrowsDocblock@14:16-14:44: Exception ValueError is thrown but not declared in @throws
MissingThrowsDocblock@18:19-18:73: Exception ValueError is thrown but not declared in @throws
