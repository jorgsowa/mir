===description===
reports named argument wrong type
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(x: 'hello'); }
//                        ^^^^^^^^^^ InvalidArgument: Argument $x of f() expects 'int', got '"hello"'
===expect===
