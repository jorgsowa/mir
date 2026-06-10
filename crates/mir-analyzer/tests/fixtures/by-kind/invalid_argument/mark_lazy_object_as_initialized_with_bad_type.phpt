===description===
Mark lazy object as initialized with bad type
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->markLazyObjectAsInitialized(new Bar);
===expect===
