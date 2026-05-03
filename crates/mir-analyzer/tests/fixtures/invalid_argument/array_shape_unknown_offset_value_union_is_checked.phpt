===description===
array shape unknown offset value union is checked
===file===
<?php
function takes_int(int $value): void { var_dump($value); }

function test(string $key): void {
    $row = ['id' => 123, 'name' => 'Ada'];
    takes_int($row[$key]);
}
===expect===
InvalidArgument: Argument $value of takes_int() expects 'int', got '123|"Ada"'
===ignore===
TODO
