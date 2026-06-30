===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a class method.
===file===
<?php
class Foo {
    #[Cache]
    public function bar(): void {}
}
===expect===
UndefinedAttributeClass@3:6-3:11: Attribute class Cache does not exist
