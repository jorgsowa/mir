===description===
Attribute invalid target property
===file===
<?php
class Foo {
    #[Attribute]
    public string $bar = "baz";
}

===expect===
InvalidAttribute@3:6-3:15: #[Attribute] can only be applied to classes, not properties
