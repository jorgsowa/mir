===description===
Override protected property access level to private
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
    protected $foo;
}

class B extends A {
    /** @var string|null */
    private $foo;
//  ^^^^^^^^^^^^^ OverriddenPropertyAccess: Access level to B::$foo must be protected (as in class A) or weaker
}
