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
UndefinedMagicPropertyFetch
===ignore===
TODO
