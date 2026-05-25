===description===
Attribute class cannot have private constructor
===file===
<?php
#[Attribute]
class Baz {
    private function __construct() {}
}
===expect===
InvalidAttribute
===ignore===
TODO
