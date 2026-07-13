===description===
FN: a property composed purely via `use Trait;` (never redeclared in the
class body) was invisible to the visibility-reduction check — only
literally-declared own_properties() were checked against the parent.
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string|null */
    public $foo;
}
trait T {
    /** @var string|null */
    protected $foo;
}
class B extends A {
    use T;
}
===expect===
OverriddenPropertyAccess@8:4-8:19: Property B::$foo overrides with less visibility
