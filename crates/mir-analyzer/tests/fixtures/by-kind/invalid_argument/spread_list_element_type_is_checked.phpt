===description===
spread list element type is checked
===config===
suppress=ForbiddenCode
===file===
<?php
function takes_ints(int ...$xs): void { var_dump($xs); }

function test(): void {
    $values = ['1', '2'];
    takes_ints(...$values);
//             ^^^^^^^^^^ InvalidArgument: Argument $xs of takes_ints() expects 'int', got '"1"'
//              ^^^^^^^^^ InvalidArgument: Argument $xs of takes_ints() expects 'int', got '"2"'
}
===expect===
