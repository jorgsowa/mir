===description===
UndefinedAttributeClass fires when an undefined attribute is placed on a class property.
===file===
<?php
class Foo {
    #[Column]
    public string $name = '';
}
===expect===
UndefinedAttributeClass@3:6-3:12: Attribute class Column does not exist
