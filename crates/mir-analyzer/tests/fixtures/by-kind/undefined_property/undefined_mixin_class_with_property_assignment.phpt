===description===
Undefined mixin class with property assignment
===file===
<?php
/** @mixin B */
//  ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
class A {}

$a = new A();
$a->foo = "bar";
