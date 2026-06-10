===description===
resetAsLazyProxyWithBadType_3
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyProxy(new Foo, fn(Foo $foo) => new Bar);
===expect===
