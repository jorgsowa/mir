===description===
Bad assignment
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
InvalidPropertyAssignment@8:9-8:23: Property $foo expects 'string', cannot assign '5'
