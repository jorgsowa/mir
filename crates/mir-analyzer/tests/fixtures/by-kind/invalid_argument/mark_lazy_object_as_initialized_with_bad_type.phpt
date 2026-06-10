===description===
Mark lazy object as initialized with bad type
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->markLazyObjectAsInitialized(new Bar);
===expect===
