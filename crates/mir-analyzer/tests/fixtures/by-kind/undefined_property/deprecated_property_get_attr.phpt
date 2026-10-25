===description===
Deprecated property get attr
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
     * @var ?int
     */
    #[Deprecated]
//    ^^^^^^^^^^ CompileError: Attribute "Deprecated" cannot target property (allowed targets: class, function, method, class constant, constant) [attribute-target-mismatch]
    public $foo;
}
echo (new A)->foo;
//            ^^^ DeprecatedProperty: Property A::$foo is deprecated
