===description===
reports null passed as int
===file===
<?php
function f(int $x): void { var_dump($x); }
function test(): void { f(null); }
===expect===
NullArgument@3:27-3:31: Argument $x of f() cannot be null
