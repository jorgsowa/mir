===description===
Deprecated property get from inside the class
===config===
suppress=MissingPropertyType
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
    }
}

===expect===
DeprecatedProperty@10:20-10:23: Property A::$foo is deprecated
