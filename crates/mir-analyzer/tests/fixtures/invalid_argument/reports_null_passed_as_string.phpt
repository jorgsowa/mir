===source===
<?php
function f(string $x): void {}
function test(): void { f(null); }
===expect===
UnusedParam: Parameter $x is never used
InvalidArgument: Argument $x of f() expects 'string', got 'null'
