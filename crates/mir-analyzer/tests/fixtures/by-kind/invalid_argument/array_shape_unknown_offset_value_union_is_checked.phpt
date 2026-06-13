===description===
array shape unknown offset value union is checked
===config===
suppress=ForbiddenCode
===file===
<?php
function takes_int(int $value): void { var_dump($value); }

function test(string $key): void {
    $row = ['id' => 123, 'name' => 'Ada'];
    takes_int($row[$key]);
}
===expect===
PossiblyInvalidArgument@6:15-6:25: Argument $value of takes_int() expects 'int', possibly different type '123|"Ada"' provided
