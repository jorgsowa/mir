===description===
reports object passed as int
===config===
suppress=ForbiddenCode
===file===
<?php
class Foo {}
function f(int $x): void { var_dump($x); }
function test(): void { f(new Foo()); }
===expect===
InvalidArgument@4:27-4:36: Argument $x of f() expects 'int', got 'Foo'
