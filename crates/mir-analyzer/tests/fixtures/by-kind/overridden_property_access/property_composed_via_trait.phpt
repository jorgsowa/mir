===description===
FN: a property composed purely via `use Trait;` (never redeclared in the
class body) was invisible to the visibility-reduction check — only
literally-declared own_properties() were checked against the parent.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var string|null */
    public $foo;
}
trait T {
    /** @var string|null */
    protected $foo;
//  ^^^^^^^^^^^^^^^ OverriddenPropertyAccess: Property B::$foo overrides with less visibility
}
class B extends A {
    use T;
}
===expect===
