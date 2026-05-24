===description===
reports incompatible union arg
===file===
<?php
function g(): int|string { return 1; }
function f(int $x): void { var_dump($x); }
function test(): void { f(g()); }
===expect===
InvalidArgument@4:27: Argument $x of f() expects 'int', got 'int|string'
