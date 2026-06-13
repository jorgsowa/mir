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
DeprecatedProperty@10:9-10:24: Property A::$foo is deprecated
