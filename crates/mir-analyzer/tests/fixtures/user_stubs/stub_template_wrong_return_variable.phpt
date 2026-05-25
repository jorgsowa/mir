===description===
stub using wrong template variable in return type produces a mismatched list type
===config===
stub_file=stubs/helpers.php
suppress=UnusedVariable,UnusedFunction
===file:stubs/helpers.php===
<?php
/**
 * @template TKey of array-key
 * @template TValue
 * @param array<TKey, TValue> $array
 * @return list<TValue>
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
    $_ = $keys;
}
===expect===
App.php: TypeCheckMismatch@8:5: Type of $keys is expected to be list<string>, got list<int>
