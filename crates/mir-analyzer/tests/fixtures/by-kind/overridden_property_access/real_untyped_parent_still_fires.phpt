===description===
A real PHP property without a type hint (has_native_type=false, from_docblock=false)
still establishes a visibility contract. Reducing visibility on such a property
must still emit OverriddenPropertyAccess.
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string|null */
    public $foo;
    /** @var int */
    protected $bar;
}

class B extends A {
    private $foo;
    private $bar;
}
===expect===
OverriddenPropertyAccess@10:4-10:17: Property B::$foo overrides with less visibility
OverriddenPropertyAccess@11:4-11:17: Property B::$bar overrides with less visibility
