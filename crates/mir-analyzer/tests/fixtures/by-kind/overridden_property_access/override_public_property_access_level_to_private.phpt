===description===
Override public property access level to private
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string|null */
    public $foo;
}

class B extends A {
    /** @var string|null */
    private $foo;
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Property B::$foo overrides with less visibility
}
===expect===
