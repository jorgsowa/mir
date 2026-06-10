===description===
resetAsLazyGhostWithBadType_1
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyGhost(new Bar, function (Foo $foo) {});
===expect===
