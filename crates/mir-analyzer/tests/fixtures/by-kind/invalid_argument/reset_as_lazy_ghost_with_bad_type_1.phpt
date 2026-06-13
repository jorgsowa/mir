===description===
resetAsLazyGhostWithBadType_1
===config===
suppress=MissingClosureReturnType
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->resetAsLazyGhost(new Bar, function (Foo $foo) {});
===expect===
