===description===
reports via variable assignment
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $s): void {
    $pos = strpos($s, 'x');
    /** @mir-check $pos is int|false */
    takesInt($pos);
}
===expect===
PossiblyInvalidArgument@6:14-6:18: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
