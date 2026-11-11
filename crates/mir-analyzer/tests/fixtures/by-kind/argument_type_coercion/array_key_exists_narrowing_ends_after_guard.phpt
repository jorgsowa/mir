===description===
array_key_exists() on a literal map narrows the key inside the guard only; after it the key is a plain string again.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 'a'|'b' $key */
function takesAB(string $key): void {}

function run(string $key): void {
    $values = ['a' => 1, 'b' => 2];
    if (array_key_exists($key, $values)) {
        takesAB($key);
    }
    takesAB($key);
//          ^^^^ ArgumentTypeCoercion: Argument $key of takesAB() expects '"a"|"b"', got 'string' — coercion may fail at runtime
}
