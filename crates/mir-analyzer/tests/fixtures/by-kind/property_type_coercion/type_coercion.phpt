===description===
Type coercion
===file===
<?php
class A {
    /** @var B|null */
    public $foo;

    public function barBar(A $a): void
    {
        $this->foo = $a;
    }
}

class B extends A {}
===expect===
InvalidPropertyAssignment@8:9-8:24: Property $foo expects 'B|null', cannot assign 'A'
