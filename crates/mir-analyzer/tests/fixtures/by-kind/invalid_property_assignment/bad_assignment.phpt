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
InvalidPropertyAssignment@8:8-8:22: Property $foo expects 'string', cannot assign '5'
