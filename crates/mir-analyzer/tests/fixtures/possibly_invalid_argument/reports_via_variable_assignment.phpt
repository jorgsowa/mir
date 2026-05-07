===description===
reports via variable assignment
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $s): void {
    $pos = strpos($s, 'x');
    takesInt($pos);
}
===expect===
PossiblyInvalidArgument@5:13: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
