===description===
Undefined mixin class with method call
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo();
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
UndefinedMethod@5:0-5:14: Method A::foo() does not exist
