===description===
undefinedMixinClassWithStaticMethodCall
===file===
<?php
/** @mixin B */
class A {}

A::foo();
===expect===
UndefinedMethod@5:0: Method A::foo() does not exist
