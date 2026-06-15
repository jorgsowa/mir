===description===
passing array<object, mixed> to @template TKey of array-key violates the bound
===config===
stub_file=stubs/helpers.php
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
/** @param array<object, int> $arr */
function test(array $arr): void {
    array_key_list($arr);
}
===expect===
App.php: InvalidTemplateParam@4:4-4:24: Template type 'TKey' inferred as 'object' does not satisfy bound 'int|string'
