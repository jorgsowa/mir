===description===
Undefined mixin class with property fetch
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo;
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
UndefinedProperty@5:9-5:12: Property A::$foo does not exist
