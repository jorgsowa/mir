===description===
reports null passed as int
===config===
suppress=ForbiddenCode
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(null); }
===expect===
NullArgument@3:26-3:30: Argument $x of f() cannot be null
