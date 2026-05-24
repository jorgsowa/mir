===description===
newLazyProxyWithBadType_1
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->newLazyProxy(fn(Bar $bar) => new Foo);
===expect===
Argument 1 of ReflectionClass::newLazyProxy expects impure-callable(Foo):Foo, but pure-Closure(Bar):Foo provided
===ignore===
TODO
