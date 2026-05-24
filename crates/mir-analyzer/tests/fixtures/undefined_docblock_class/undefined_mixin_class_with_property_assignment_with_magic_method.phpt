===description===
undefinedMixinClassWithPropertyAssignment_WithMagicMethod
===file===
<?php
/**
 * @property string $baz
 * @mixin B
 */
class A {
    public function __set(string $name, string $value) {}
}

(new A)->foo = "bar";
===expect===
UndefinedDocblockClass
===ignore===
TODO
