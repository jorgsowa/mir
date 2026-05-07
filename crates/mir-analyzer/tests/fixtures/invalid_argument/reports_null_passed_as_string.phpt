===description===
reports null passed as string
===file===
<?php
function f(string $x): void { var_dump($x); }
function test(): void { f(null); }
===expect===
NullArgument@3:26: Argument $x of f() cannot be null
