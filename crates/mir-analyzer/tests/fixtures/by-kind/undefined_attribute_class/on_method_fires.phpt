===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a class method.
===file===
<?php
class Foo {
    #[Cache]
//    ^^^^^ UndefinedAttributeClass: Attribute class Cache does not exist
    public function bar(): void {}
}
===expect===
