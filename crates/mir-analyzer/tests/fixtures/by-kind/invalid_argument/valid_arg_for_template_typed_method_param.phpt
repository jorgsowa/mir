===description===
passing correct type to a method whose param is the class template param should not error
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
    $box->set(42);
}
===expect===
