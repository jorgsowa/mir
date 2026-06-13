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
DeprecatedProperty@9:15-9:18: Property A::$foo is deprecated
