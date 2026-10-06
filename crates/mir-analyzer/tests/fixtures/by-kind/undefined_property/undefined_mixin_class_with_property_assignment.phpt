===description===
Undefined mixin class with property assignment
===file===
<?php
/** @mixin B */
//  ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
class A {}

(new A)->foo = "bar";
===expect===
