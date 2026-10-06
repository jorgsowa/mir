===description===
Deprecated property get from inside the class
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
    public function bar(): void
    {
        echo $this->foo;
//                  ^^^ DeprecatedProperty: Property A::$foo is deprecated
    }
}
