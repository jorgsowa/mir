===description===
Get lazy initializer with bad type
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->getLazyInitializer(new Bar);
===expect===
