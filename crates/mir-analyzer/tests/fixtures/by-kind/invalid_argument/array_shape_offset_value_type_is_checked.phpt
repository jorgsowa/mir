===description===
array shape offset value type is checked
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takes_string(string $s): void { var_dump($s); }

function test(): void {
    $row = ['id' => 123, 'name' => 'Ada'];
    takes_string($row['id']);
//               ^^^^^^^^^^ ArgumentTypeCoercion: Argument $s of takes_string() expects 'string', got '123' — coercion may fail at runtime
}
===expect===
