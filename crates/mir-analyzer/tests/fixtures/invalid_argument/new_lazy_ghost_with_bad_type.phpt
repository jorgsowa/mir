===description===
New lazy ghost with bad type
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->newLazyGhost(function (Bar $foo) {});
===expect===
Argument 1 of ReflectionClass::newLazyGhost expects impure-callable(Foo):void, but pure-Closure(Bar):void provided
===ignore===
TODO
