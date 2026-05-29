===description===
passing wrong type to an inherited method whose param is the parent's template param should error
===file===
<?php
/** @template T */
class Box {
    private mixed $value = null;
    /** @param T $value */
    public function set(mixed $value): void { $this->value = $value; }
}
/** @extends Box<int> */
class IntBox extends Box {}
function test(): void {
    $box = new IntBox();
    $box->set("hello");
}
===expect===
InvalidArgument@12:15-12:22: Argument $value of set() expects 'int', got '"hello"'
