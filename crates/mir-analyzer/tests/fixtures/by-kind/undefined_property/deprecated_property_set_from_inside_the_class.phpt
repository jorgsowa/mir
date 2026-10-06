===description===
Deprecated property set from inside the class
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A{
    /**
     * @deprecated
     * @var ?int
     */
    public $foo;
    public function bar(int $p): void
    {
        $this->foo = $p;
//      ^^^^^^^^^^^^^^^ DeprecatedProperty: Property A::$foo is deprecated
    }
}
