===description===
reports string passed as int
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f('hello'); }
===expect===
InvalidArgument@3:26: Argument $x of f() expects 'int', got '"hello"'
===ignore===
TODO
