===description===
resetAsLazyGhostWithBadType_2
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyGhost(new Foo, function (Bar $foo) {});
===expect===
Argument 2 of ReflectionClass::resetAsLazyGhost expects impure-callable(Foo):void, but pure-Closure(Bar):void provided
===ignore===
TODO
