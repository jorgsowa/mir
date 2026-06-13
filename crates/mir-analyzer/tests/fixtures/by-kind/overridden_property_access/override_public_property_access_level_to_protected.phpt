===description===
Override public property access level to protected
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
    protected $foo;
}
===expect===
OverriddenPropertyAccess@9:4-9:19: Property B::$foo overrides with less visibility
