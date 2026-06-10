===description===
resetAsLazyProxyWithBadType_2
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyProxy(new Foo, fn(Bar $bar) => new Foo);
===expect===
