===description===
a helpers file present on disk but not registered via stub_file= is analyzed as a
regular source file — its symbols are available but its body is checked for errors
===config===
suppress=UnusedVariable,UnusedFunction
===file:stubs/helpers.php===
<?php
/**
 * @template TKey of array-key
 * @template TValue
 * @param array<TKey, TValue> $array
 * @return list<TKey>
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
helpers.php: UnusedParam@8:25-8:37: Parameter $array is never used
