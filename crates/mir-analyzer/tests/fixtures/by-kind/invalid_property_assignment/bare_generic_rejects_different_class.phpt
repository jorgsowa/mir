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
InvalidPropertyAssignment@14:9: Property $item expects 'BoxA', cannot assign 'BoxB<string>'
