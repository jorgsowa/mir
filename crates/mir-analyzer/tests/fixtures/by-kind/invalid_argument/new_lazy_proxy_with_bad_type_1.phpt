===description===
newLazyProxyWithBadType_1
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->newLazyProxy(fn(Bar $bar) => new Foo);
===expect===
