===description===
Override public property access level to private
===file===
<?php
class A {
    /** @var string|null */
    public $foo;
}

class B extends A {
    /** @var string|null */
    private $foo;
}
===expect===
OverriddenPropertyAccess@9:4-9:17: Property B::$foo overrides with less visibility
