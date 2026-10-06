===description===
undefinedMixinClassWithPropertyAssignment_WithMagicMethod
===file===
<?php
/**
 * @property string $baz
 * @mixin B
// ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
 */
class A {
    public function __set(string $name, string $value) {}
}

(new A)->foo = "bar";
