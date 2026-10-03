===description===
stub with @template TKey returns list<TKey> inferred from typed array<string, int> parameter
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
