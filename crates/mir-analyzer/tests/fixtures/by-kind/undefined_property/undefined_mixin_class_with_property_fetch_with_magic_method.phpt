===description===
undefinedMixinClassWithPropertyFetch_WithMagicMethod
===ignore===
TODO
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
