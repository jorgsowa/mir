===description===
Undefined mixin class with method call
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo();
===expect===
UndefinedMethod@5:1: Method A::foo() does not exist
