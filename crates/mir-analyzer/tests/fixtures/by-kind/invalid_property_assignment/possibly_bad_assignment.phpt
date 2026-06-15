===description===
Possibly bad assignment
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var string */
    public $foo;

    public function barBar(): void
    {
        $this->foo = rand(0, 1) ? 5 : "hello";
    }
}
===expect===
InvalidPropertyAssignment@8:8-8:45: Property $foo expects 'string', cannot assign '5|"hello"'
