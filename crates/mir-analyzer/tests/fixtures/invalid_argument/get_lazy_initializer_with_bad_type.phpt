===description===
getLazyInitializerWithBadType
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->getLazyInitializer(new Bar);
===expect===
Argument 1 of ReflectionClass::getLazyInitializer expects Foo, but Bar provided
===ignore===
TODO
