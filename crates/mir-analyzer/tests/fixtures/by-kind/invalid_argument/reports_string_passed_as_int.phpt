===description===
reports string passed as int
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f('hello'); }
===expect===
InvalidArgument@3:26-3:33: Argument $x of f() expects 'int', got '"hello"'
