===description===
does not report mixed arg
===config===
suppress=ForbiddenCode,MixedArgument
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(mixed $v): void { f($v); }
===expect===
