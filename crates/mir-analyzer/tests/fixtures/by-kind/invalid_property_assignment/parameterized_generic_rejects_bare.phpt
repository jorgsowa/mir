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
InvalidPropertyAssignment@8:1: Property $item expects 'Box<string>', cannot assign 'Box'
