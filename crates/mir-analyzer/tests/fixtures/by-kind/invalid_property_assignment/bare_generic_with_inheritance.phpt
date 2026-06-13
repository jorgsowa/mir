===description===
bare generic property accepts parameterized subclass
===file===
<?php
/** @template T */
class Base {}

class Derived extends Base {}

class Holder {
    private Base $item;

    public function assign(): void {
        /** @var Derived<string> $d */
        $d = new Derived();
        $this->item = $d;
    }
}
===expect===
MissingConstructor@7:0-7:14: Class Holder has uninitialized properties but no constructor
