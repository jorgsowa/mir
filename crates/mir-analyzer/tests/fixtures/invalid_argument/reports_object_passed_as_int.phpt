===file===
<?php
class Foo {}
function f(int $x): void { var_dump($x); }
function test(): void { f(new Foo()); }
===expect===
InvalidArgument: Argument $x of f() expects 'int', got 'Foo'
