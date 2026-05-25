===description===
stub with @template TKey returns list<TKey> inferred from typed array<string, int> parameter
===config===
stub_file=stubs/helpers.php
suppress=UnusedVariable,UnusedFunction
===file:stubs/helpers.php===
<?php
/**
 * @template TKey of array-key
 * @template TValue
 * @param array<TKey, TValue> $array
 * @phpstan-return list<TKey>
 */
function array_key_list(array $array): array {}
===file:App.php===
<?php
/**
 * @param array<string, int> $arr
 */
function test(array $arr): void {
    $keys = array_key_list($arr);
    /** @mir-check $keys is list<string> */
}
===expect===
