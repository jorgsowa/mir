===description===
resetAsLazyProxyWithBadType_1
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyProxy(new Bar, fn(Foo $foo) => new Foo);
===expect===
Argument 1 of ReflectionClass::resetAsLazyProxy expects Foo, but Bar provided
===ignore===
TODO
