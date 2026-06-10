===description===
newLazyProxyWithBadType_2
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->newLazyProxy(fn(Foo $foo) => new Bar);
===expect===
