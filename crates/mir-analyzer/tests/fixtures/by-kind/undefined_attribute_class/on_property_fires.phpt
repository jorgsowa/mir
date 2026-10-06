===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a class property.
===file===
<?php
class Foo {
    #[Column]
//    ^^^^^^ UndefinedAttributeClass: Attribute class Column does not exist
    public string $name = '';
}
