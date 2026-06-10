===description===
New lazy ghost with bad type
===ignore===
TODO
===file===
<?php
class Foo {}
class Bar {}
$reflectionClass = new ReflectionClass(Foo::class);
$reflectionClass->newLazyGhost(function (Bar $foo) {});
===expect===
