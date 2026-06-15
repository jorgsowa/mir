===description===
Deprecated property set from inside the class
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
    public function bar(int $p): void
    {
        $this->foo = $p;
    }
}

===expect===
DeprecatedProperty@10:8-10:23: Property A::$foo is deprecated
