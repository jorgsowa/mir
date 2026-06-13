===description===
Override protected property access level to private
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string|null */
    protected $foo;
}

class B extends A {
    /** @var string|null */
    private $foo;
}
===expect===
OverriddenPropertyAccess@9:4-9:17: Property B::$foo overrides with less visibility
