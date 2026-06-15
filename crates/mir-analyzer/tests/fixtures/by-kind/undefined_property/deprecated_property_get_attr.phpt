===description===
Deprecated property get attr
===config===
suppress=MissingPropertyType
===file===
<?php
class A{
    /**
     * @var ?int
     */
    #[Deprecated]
    public $foo;
}
echo (new A)->foo;
===expect===
DeprecatedProperty@9:14-9:17: Property A::$foo is deprecated
