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
}
===expect===
InvalidArgument@6:15-6:25: Argument $xs of takes_ints() expects 'int', got '"1"'
InvalidArgument@6:16-6:25: Argument $xs of takes_ints() expects 'int', got '"2"'
