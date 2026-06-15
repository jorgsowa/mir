===description===
Undefined mixin class with static method call
===file===
<?php
/** @mixin B */
class A {}

A::foo();
===expect===
UndefinedMethod@5:0-5:8: Method A::foo() does not exist
