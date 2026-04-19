===source===
<?php
function g(): int|string { return 1; }
function f(int $x): void { var_dump($x); }
function test(): void { f(g()); }
===expect===
InvalidArgument: Argument $x of f() expects 'int', got 'int|string'
