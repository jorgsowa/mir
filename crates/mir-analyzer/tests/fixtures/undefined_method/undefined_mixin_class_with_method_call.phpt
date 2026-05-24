===description===
undefinedMixinClassWithMethodCall
===file===
<?php
/** @mixin B */
class A {}

(new A)->foo();
===expect===
UndefinedMethod@5:0: Method A::foo() does not exist
