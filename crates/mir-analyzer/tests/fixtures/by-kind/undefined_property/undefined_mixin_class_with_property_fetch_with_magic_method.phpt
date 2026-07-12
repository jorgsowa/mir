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
UndefinedDocblockClass@2:0-5:3: Docblock type 'B' does not exist
