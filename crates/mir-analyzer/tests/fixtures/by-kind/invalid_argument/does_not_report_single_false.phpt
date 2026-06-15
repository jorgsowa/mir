===description===
does not report single false
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(): void {
    takesInt(false);
}
===expect===
InvalidArgument@4:13-4:18: Argument $n of takesInt() expects 'int', got 'false'
