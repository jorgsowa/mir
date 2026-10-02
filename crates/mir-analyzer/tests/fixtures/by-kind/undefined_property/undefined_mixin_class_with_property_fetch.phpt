===description===
Undefined mixin class with property fetch
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo;
//       ^^^ UndefinedProperty: Property A::$foo does not exist
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
