===description===
reports strpos result passed to int param
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $haystack, string $needle): void {
    takesInt(strpos($haystack, $needle));
}
===expect===
PossiblyInvalidArgument@4:14: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
