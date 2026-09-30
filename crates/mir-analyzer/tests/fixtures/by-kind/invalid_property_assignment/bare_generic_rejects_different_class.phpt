===description===
bare generic property still rejects value of different class, even if parameterized
===file===
<?php
/** @template T */
class BoxA {}

/** @template T */
class BoxB {}

class Holder {
//<^^^^^^^^^^^^^^ MissingConstructor: Class Holder has uninitialized properties but no constructor
    private BoxA $item;

    public function bad(): void {
        /** @var BoxB<string> $b */
        $b = new BoxB();
        $this->item = $b;
//      ^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $item expects 'BoxA', cannot assign 'BoxB<string>'
    }
}
===expect===
