===description===
mixedTypeCoercion
===file===
<?php
class A {
    /** @var array<int, A> */
    public $foo = [];

    /** @param A[] $arr */
    public function barBar(array $arr): void
    {
        $this->foo = $arr;
    }
}
===expect===
MixedPropertyTypeCoercion
===ignore===
TODO
