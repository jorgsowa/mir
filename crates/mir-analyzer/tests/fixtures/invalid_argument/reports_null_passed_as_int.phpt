===source===
<?php
function f(int $x): void {}
function test(): void { f(null); }
===expect===
UnusedParam: Parameter $x is never used
InvalidArgument: Argument $x of f() expects 'int', got 'null'
