===description===
passing array<object, mixed> to @template TKey of array-key violates the bound
===config===
<mir>
  <stubs>
    <file name="stubs/helpers.php"/>
  </stubs>
</mir>
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
//  ^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'TKey' inferred as 'object' does not satisfy bound 'int|string'
}
