===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(null); }
===expect===
NullArgument: Argument $x of f() cannot be null
