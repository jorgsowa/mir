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
UndefinedDocblockClass@2:0-5:3: Docblock type 'B' does not exist
