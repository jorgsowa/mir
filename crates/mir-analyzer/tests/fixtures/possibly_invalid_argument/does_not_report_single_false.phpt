===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(): void {
    takesInt(false);
}
===expect===
InvalidArgument: Argument $n of takesInt() expects 'int', got 'false'
