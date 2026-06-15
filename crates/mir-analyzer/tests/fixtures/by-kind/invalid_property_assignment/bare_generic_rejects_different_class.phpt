===description===
bare generic property still rejects value of different class, even if parameterized
===file===
<?php
/** @template T */
class BoxA {}

/** @template T */
class BoxB {}

class Holder {
    private BoxA $item;

    public function bad(): void {
        /** @var BoxB<string> $b */
        $b = new BoxB();
        $this->item = $b;
    }
}
===expect===
MissingConstructor@8:0-8:14: Class Holder has uninitialized properties but no constructor
InvalidPropertyAssignment@14:8-14:24: Property $item expects 'BoxA', cannot assign 'BoxB<string>'
