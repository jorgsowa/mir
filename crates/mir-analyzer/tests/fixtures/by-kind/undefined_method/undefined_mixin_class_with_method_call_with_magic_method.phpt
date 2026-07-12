===description===
undefinedMixinClassWithMethodCall_WithMagicMethod
===file===
<?php
/**
 * @method baz()
 * @mixin B
 */
class A {
    public function __call(string $name, array $arguments) {}
}

(new A)->foo();
===expect===
UndefinedDocblockClass@2:0-5:3: Docblock type 'B' does not exist
