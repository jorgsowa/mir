===description===
Possibly bad assignment
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
MissingConstructor@2:0-2:9: Class A has uninitialized properties but no constructor
InvalidPropertyAssignment@8:9-8:46: Property $foo expects 'string', cannot assign '5|"hello"'
