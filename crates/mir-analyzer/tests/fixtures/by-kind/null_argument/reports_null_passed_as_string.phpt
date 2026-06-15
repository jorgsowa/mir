===description===
reports null passed as string
===config===
suppress=ForbiddenCode
===file===
<?php
function f(string $x): void { var_dump($x); }
function test(): void { f(null); }
===expect===
NullArgument@3:26-3:30: Argument $x of f() cannot be null
