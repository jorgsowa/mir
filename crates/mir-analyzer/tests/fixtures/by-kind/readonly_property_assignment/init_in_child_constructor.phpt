===description===
ReadonlyPropertyAssignment when a subclass's own constructor writes to a readonly property declared on the parent.
===config===
suppress=MissingConstructor
===file===
<?php
class Base {
    public readonly string $name;
}

class Child extends Base {
    public function __construct(string $name) {
        $this->name = $name;
    }
}
===expect===
ReadonlyPropertyAssignment@8:8-8:27: Cannot assign to readonly property Child::$name outside of constructor
