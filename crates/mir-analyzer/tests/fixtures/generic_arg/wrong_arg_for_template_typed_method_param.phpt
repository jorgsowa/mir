===description===
passing wrong type to a method whose param is declared as the class template param should error
===file===
<?php
/** @template T */
class Box {
    private mixed $value = null;
    /** @param T $value */
    public function set(mixed $value): void { $this->value = $value; }
}
function test(): void {
    /** @var Box<int> $box */
    $box = new Box();
    $box->set("hello");
}
===expect===
InvalidArgument@11:14: Argument $value of set() expects 'int', got '"hello"'
