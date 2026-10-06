===description===
Undefined mixin class with property fetch
===file===
<?php
/** @mixin B */
//  ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
class A {}

(new A)->foo;
//       ^^^ UndefinedProperty: Property A::$foo does not exist
===expect===
