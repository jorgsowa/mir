===description===
undefinedMixinClassWithStaticMethodCall_WithMagicMethod
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
UndefinedDocblockClass@2:0-5:3: Docblock type 'B' does not exist
