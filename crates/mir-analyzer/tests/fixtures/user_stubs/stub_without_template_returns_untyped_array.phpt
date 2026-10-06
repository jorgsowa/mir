===description===
stub without @template annotations returns array<mixed, mixed>, not a typed list
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
function array_key_list(array $array): array {}
===file:App.php===
<?php
function test(): void {
    $keys = array_key_list(['x' => 1, 'y' => 2]);
    /** @mir-check $keys is list<string> */
    $_ = $keys;
//  ^^^^^^^^^^^ TypeCheckMismatch: Type of $keys is expected to be list<string>, got array
}
