===description===
bare generic property accepts bare generic value (no parameterization)
===file===
<?php
/** @template T */
class Item {}

class Holder {
//<^^^^^^^^^^^^^^ MissingConstructor: Class Holder has uninitialized properties but no constructor
    private Item $value;

    public function set(): void {
        $item = new Item();
        $this->value = $item;
    }
}
===expect===
