===description===
reports null passed as int
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(null); }
//                        ^^^^ NullArgument: Argument $x of f() cannot be null
===expect===
