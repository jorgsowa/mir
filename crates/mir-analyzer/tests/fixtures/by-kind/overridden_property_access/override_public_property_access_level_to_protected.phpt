===description===
Override public property access level to protected
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
OverriddenPropertyAccess
===ignore===
TODO
