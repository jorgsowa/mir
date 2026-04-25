===file===
<?php
function takes_ints(int ...$xs): void { var_dump($xs); }

function test(): void {
    $values = ['1', '2'];
    takes_ints(...$values);
}
===expect===
InvalidArgument: Argument $xs of takes_ints() expects 'int', got '"1"|"2"'
