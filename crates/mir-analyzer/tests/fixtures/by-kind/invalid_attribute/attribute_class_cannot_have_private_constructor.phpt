===description===
Attribute class cannot have private constructor
===file===
<?php
#[Attribute]
class Baz {
    private function __construct() {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidAttribute: Attribute class constructor must not be private
}
===expect===
