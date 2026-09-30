===description===
reports string passed as int
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f('hello'); }
//                        ^^^^^^^ InvalidArgument: Argument $x of f() expects 'int', got '"hello"'
===expect===
