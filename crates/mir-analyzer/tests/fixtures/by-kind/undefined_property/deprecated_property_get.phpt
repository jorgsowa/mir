===description===
Deprecated property get
===file===
<?php
class A{
    /**
     * @deprecated
     * @var ?int
     */
    public $foo;
}
echo (new A)->foo;
===expect===
DeprecatedProperty@9:15-9:18: Property A::$foo is deprecated
