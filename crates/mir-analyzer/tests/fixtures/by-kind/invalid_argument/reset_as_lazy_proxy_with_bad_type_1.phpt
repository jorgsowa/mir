===description===
resetAsLazyProxyWithBadType_1
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyProxy(new Bar, fn(Foo $foo) => new Foo);
===expect===
