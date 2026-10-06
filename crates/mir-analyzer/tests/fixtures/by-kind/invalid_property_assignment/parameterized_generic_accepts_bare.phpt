===description===
A bare (unbound) generic value is `Box<mixed>`, so a parameterized property accepts it;
a value bound to a different type still errors.
===file===
<?php
/** @template T */
class Box {
    /** @param T $v */
    public function __construct(public mixed $v = null) {}
}

class Holder {
//<^^^^^^^^^^^^^^ MissingConstructor: Class Holder has uninitialized properties but no constructor
    /** @var Box<string> */
    private Box $item;

    public function ok(): void {
        $item = new Box();
        $this->item = $item;
    }

    public function bad(): void {
        $this->item = new Box(1);
//      ^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $item expects 'Box<string>', cannot assign 'Box<int>'
    }
}
