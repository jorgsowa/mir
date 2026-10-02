===description===
Undefined mixin class with method call
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo();
//<^^^^^^^^^^^^^^ UndefinedMethod: Method A::foo() does not exist
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
