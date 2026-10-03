===description===
array_column with a dynamic $index_key is not necessarily a list and keeps the stub's array type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $rows, string $idx): void {
    $keyed = array_column($rows, 'name', $idx);
    /** @mir-check $keyed is array<array-key, mixed> */
    $_ = $keyed;

    $keyed_literal = array_column($rows, 'name', 'id');
    /** @mir-check $keyed_literal is array<array-key, mixed> */
    $_ = $keyed_literal;
}
===expect===
