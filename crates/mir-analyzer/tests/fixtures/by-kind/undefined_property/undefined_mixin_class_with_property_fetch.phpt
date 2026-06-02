===description===
Undefined mixin class with property fetch
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo;
===expect===
UndefinedProperty@5:10-5:13: Property A::$foo does not exist
