===description===
Undefined mixin class with static method call
===file===
<?php
/** @mixin B */
class A {}

A::foo();
===expect===
UndefinedDocblockClass@2:0-2:15: Docblock type 'B' does not exist
UndefinedMethod@5:0-5:8: Method A::foo() does not exist
