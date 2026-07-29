===description===
NoInterfaceProperties fires on a property write through a plain interface type
without @seal-properties too — the write-side check (expr/assignment.rs) mirrors
the read-side one.
===config===
suppress=MixedAssignment
===file===
<?php
interface Shape {
    public function area(): float;
}

class Circle implements Shape {
    public float $radius = 1.0;
    public function area(): float {
        return 3.14 * $this->radius * $this->radius;
    }
}

function resize(Shape $s, float $value): void {
    $s->radius = $value;
}

===expect===
NoInterfaceProperties@14:4-14:23: Property $radius is not defined on this interface
