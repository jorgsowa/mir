===description===
Attribute invalid target property
===file===
<?php
class Foo {
    #[Attribute]
    public string $bar = "baz";
}

===expect===
InvalidAttribute
===ignore===
TODO
