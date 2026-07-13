===description===
ReadonlyPropertyAssignment when child class method tries to set parent's readonly property
===config===
suppress=MissingConstructor
===file===
<?php
class Base {
    public readonly string $name;
}

class Child extends Base {
    public function init(string $name): void {
        $this->name = $name;
    }
}
===expect===
ReadonlyPropertyAssignment@8:8-8:27: Cannot assign to readonly property Base::$name outside of constructor
