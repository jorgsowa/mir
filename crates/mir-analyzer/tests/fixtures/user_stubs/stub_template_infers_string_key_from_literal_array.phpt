===description===
stub with @template TKey returns list<TKey> inferred as list<string> from string-keyed literal array
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
function test(): void {
    $keys = array_key_list(['x' => 1, 'y' => 2]);
    /** @mir-check $keys is list<string> */
}
===expect===
