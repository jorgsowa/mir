===description===
stub with @template TKey returns list<TKey> inferred as list<string> from string-keyed literal array
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
function test(): void {
    $keys = array_key_list(['x' => 1, 'y' => 2]);
    /** @mir-check $keys is list<string> */
    $_ = $keys;
}
