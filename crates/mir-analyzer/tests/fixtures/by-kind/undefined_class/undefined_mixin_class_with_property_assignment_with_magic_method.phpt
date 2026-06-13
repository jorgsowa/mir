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
MissingConstructor@6:0-6:9: Class A has uninitialized properties but no constructor
