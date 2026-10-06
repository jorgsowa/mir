===description===
undefinedMixinClassWithPropertyFetch_WithMagicMethod
===file===
<?php
/**
 * @property string $baz
 * @mixin B
// ^^^^^^^^ UndefinedDocblockClass: Docblock type 'B' does not exist
 */
class A {
    public function __get(string $name): string {
        return "";
    }
}

(new A)->foo;
===expect===
