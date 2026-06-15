===description===
reports incompatible union arg
===config===
suppress=ForbiddenCode
===file===
<?php
function g(): int|string { return 1; }
function f(int $x): void { var_dump($x); }
function test(): void { f(g()); }
===expect===
PossiblyInvalidArgument@4:26-4:29: Argument $x of f() expects 'int', possibly different type 'int|string' provided
