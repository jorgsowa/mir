===description===
Undefined mixin class with method call
===file===
<?php
/** @mixin B */
//  ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
class A {}

(new A)->foo();
//<^^^^^^^^^^^^^^ UndefinedMethod: Method A::foo() does not exist
