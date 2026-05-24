===description===
does not report single false
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(): void {
    takesInt(false);
}
===expect===
InvalidArgument@4:14: Argument $n of takesInt() expects 'int', got 'false'
