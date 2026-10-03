===description===
stub using wrong template variable in return type produces a mismatched list type
===config===
<mir>
  <stubs>
    <file name="stubs/helpers.php"/>
  </stubs>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^^^ TypeCheckMismatch: Type of $keys is expected to be list<string>, got list<int>
}
===expect===
