===description===
Undefined mixin class with property assignment
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo = "bar";
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
