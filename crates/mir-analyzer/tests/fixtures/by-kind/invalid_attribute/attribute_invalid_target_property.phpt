===description===
Attribute invalid target property
===file===
<?php
class Foo {
    #[Attribute]
    public string $bar = "baz";
}

===expect===
InvalidAttribute@3:7-3:16: #[Attribute] can only be applied to classes, not properties
