===description===
Template bound to non-empty-array<K, V> param type correctly infers V from argument
===config===
stub_file=stubs/helpers.php
suppress=UnusedVariable,UnusedFunction
===file:stubs/helpers.php===
<?php
/**
 * @template K of array-key
 * @template V
 * @param non-empty-array<K, V> $array
 * @return V
 */
function first_value(array $array): mixed {}
===file:App.php===
<?php
/**
 * @param non-empty-array<string, int> $arr
 */
function test(array $arr): void {
    $val = first_value($arr);
    /** @mir-check $val is int */
    $_ = $val;
}
===expect===
