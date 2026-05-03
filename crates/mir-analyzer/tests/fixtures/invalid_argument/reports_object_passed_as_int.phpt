===description===
reports object passed as int
===file===
<?php
class Foo {}
function f(int $x): void { var_dump($x); }
function test(): void { f(new Foo()); }
===expect===
InvalidArgument@4:26: Argument $x of f() expects 'int', got 'Foo'
===ignore===
TODO
