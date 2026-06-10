===description===
undefinedMixinClassWithMethodCall_WithMagicMethod
===ignore===
TODO
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
