===description===
resetAsLazyGhostWithBadType_2
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyGhost(new Foo, function (Bar $foo) {});
===expect===
