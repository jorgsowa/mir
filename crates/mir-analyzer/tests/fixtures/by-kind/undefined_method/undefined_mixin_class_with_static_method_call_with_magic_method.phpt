===description===
undefinedMixinClassWithStaticMethodCall_WithMagicMethod
===file===
<?php
/**
 * @method baz()
 * @mixin B
// ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
 */
class A {
    public static function __callStatic(string $name, array $arguments) {}
}

A::foo();
