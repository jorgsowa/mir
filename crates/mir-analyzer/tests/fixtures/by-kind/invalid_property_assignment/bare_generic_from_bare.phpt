===description===
bare generic property accepts bare generic value (no parameterization)
===file===
<?php
/** @template T */
class Item {}

class Holder {
    private Item $value;

    public function set(): void {
        $item = new Item();
        $this->value = $item;
    }
}
===expect===
MissingConstructor@5:0-5:14: Class Holder has uninitialized properties but no constructor
