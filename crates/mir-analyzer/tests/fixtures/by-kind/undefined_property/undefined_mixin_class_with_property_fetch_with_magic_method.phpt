===description===
undefinedMixinClassWithPropertyFetch_WithMagicMethod
===file===
<?php
/**
 * @property string $baz
 * @mixin B
 */
class A {
    public function __get(string $name): string {
        return "";
    }
}

(new A)->foo;
===expect===
MissingConstructor@6:0-6:9: Class A has uninitialized properties but no constructor
