===description===
Attribute class cannot have private constructor
===file===
<?php
#[Attribute]
class Baz {
    private function __construct() {}
}
===expect===
InvalidAttribute@4:4-4:37: Attribute class constructor must not be private
