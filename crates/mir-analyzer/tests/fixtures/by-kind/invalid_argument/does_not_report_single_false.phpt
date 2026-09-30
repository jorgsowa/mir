===description===
does not report single false
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(): void {
    takesInt(false);
//           ^^^^^ InvalidArgument: Argument $n of takesInt() expects 'int', got 'false'
}
===expect===
