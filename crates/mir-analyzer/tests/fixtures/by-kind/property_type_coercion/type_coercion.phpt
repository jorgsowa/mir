===description===
Type coercion
===config===
suppress=MissingPropertyType
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
PropertyTypeCoercion@8:9-8:24: Property $foo expects 'B|null', cannot assign 'A' — coercion may fail at runtime
