===description===
reports strpos result passed to int param
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
function test(string $haystack, string $needle): void {
    takesInt(strpos($haystack, $needle));
}
===expect===
PossiblyInvalidArgument@4:14-4:40: Argument $n of takesInt() expects 'int', possibly different type 'int<0, max>|false' provided
