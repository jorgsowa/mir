===description===
parameterized property still rejects bare generic value (invariant check)
===file===
<?php
/** @template T */
class Box {}

class Holder {
    /** @var Box<string> */
    private Box $item;

    public function bad(): void {
        $item = new Box();
        $this->item = $item;
    }
}
===expect===
MissingConstructor@5:0-5:14: Class Holder has uninitialized properties but no constructor
InvalidPropertyAssignment@11:8-11:27: Property $item expects 'Box<string>', cannot assign 'Box'
