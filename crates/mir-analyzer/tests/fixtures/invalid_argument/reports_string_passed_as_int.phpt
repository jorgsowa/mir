===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f('hello'); }
===expect===
InvalidArgument: Argument $x of f() expects 'int', got '"hello"'
