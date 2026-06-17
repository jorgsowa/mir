===description===
array shape offset value type is checked
===config===
suppress=ForbiddenCode
===file===
<?php
function takes_string(string $s): void { var_dump($s); }

function test(): void {
    $row = ['id' => 123, 'name' => 'Ada'];
    takes_string($row['id']);
}
===expect===
ArgumentTypeCoercion@6:17-6:27: Argument $s of takes_string() expects 'string', got '123' — coercion may fail at runtime
