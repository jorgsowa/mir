===description===
New lazy ghost with bad type
===config===
suppress=MissingClosureReturnType
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->newLazyGhost(function (Bar $foo) {});
===expect===
