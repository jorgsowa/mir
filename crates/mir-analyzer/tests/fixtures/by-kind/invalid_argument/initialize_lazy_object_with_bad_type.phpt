===description===
Initialize lazy object with bad type
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->initializeLazyObject(new Bar);
===expect===
