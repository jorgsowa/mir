===description===
Is uninitialized lazy object with bad type
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->isUninitializedLazyObject(new Bar);
===expect===
