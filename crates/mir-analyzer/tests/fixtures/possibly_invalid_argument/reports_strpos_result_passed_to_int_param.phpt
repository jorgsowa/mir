===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $haystack, string $needle): void {
    takesInt(strpos($haystack, $needle));
}
===expect===
PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
