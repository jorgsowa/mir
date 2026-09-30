===description===
Attribute invalid target property
===file===
<?php
class Foo {
    #[Attribute]
//    ^^^^^^^^^ InvalidAttribute: #[Attribute] can only be applied to classes, not properties
    public string $bar = "baz";
}

===expect===
