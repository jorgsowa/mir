===description===
Bad assignment
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string */
    public $foo;

    public function barBar(): void
    {
        $this->foo = 5;
    }
}
===expect===
MissingConstructor@2:0-2:9: Class A has uninitialized properties but no constructor
InvalidPropertyAssignment@8:9-8:23: Property $foo expects 'string', cannot assign '5'
