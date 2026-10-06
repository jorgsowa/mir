===description===
undefinedMixinClassWithMethodCall_WithMagicMethod
===file===
<?php
/**
 * @method baz()
 * @mixin B
// ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
 */
class A {
    public function __call(string $name, array $arguments) {}
}

(new A)->foo();
===expect===
