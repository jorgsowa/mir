===description===
spread list element type is checked
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takes_ints(int ...$xs): void { var_dump($xs); }

function test(): void {
    $values = ['1', '2'];
    takes_ints(...$values);
//             ^^^^^^^^^^ InvalidScalarArgument: Argument $xs of takes_ints() expects 'int', got scalar '"1"'
//              ^^^^^^^^^ InvalidScalarArgument: Argument $xs of takes_ints() expects 'int', got scalar '"2"'
}
