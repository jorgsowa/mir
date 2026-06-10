===description===
resetAsLazyGhostWithBadType_1
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyGhost(new Bar, function (Foo $foo) {});
===expect===
