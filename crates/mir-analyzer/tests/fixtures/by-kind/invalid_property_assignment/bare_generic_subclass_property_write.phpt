===description===
Writing a value whose bound isn't structurally provable against an
inherited `@var T` (`T of Named`) property must not be flagged from within
a bare subclass (`class IntBox extends Box {}`, no own `@template`) any
more than it already isn't flagged inside the declaring class itself —
`$this` in both cases carries no concrete type args, so T stays
unresolved. A receiver with concrete type args must still catch a real
mismatch.
===config===
suppress=UnusedParam,MissingPropertyType
===file===
<?php
interface Named {}
class Impl implements Named {}

/** @template T of Named */
class Box {
    /** @var T */
    public $item;

    public function setInBase(Impl $val): void {
        $this->item = $val;
    }
}

class IntBox extends Box {
    public function setInSub(Impl $val): void {
        $this->item = $val;
    }
}

/** @param Box<Impl> $b */
function assign_wrong_type(Box $b): void {
    $b->item = 'x';
}
===expect===
InvalidPropertyAssignment@23:4-23:18: Property $item expects 'Impl', cannot assign '"x"'
