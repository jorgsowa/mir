===description===
undefinedMixinClassWithStaticMethodCall_WithMagicMethod
===ignore===
TODO
===file===
<?php
/**
 * @method baz()
 * @mixin B
 */
class A {
    public static function __callStatic(string $name, array $arguments) {}
}

A::foo();
===expect===
