===description===
`$this->counter++` mutates a readonly property just as much as a plain
assignment does, but only `assign_to_target`'s own PropertyAccess arm ever
checked `is_readonly` -- `++`/`--` routed through a separate purity-only
helper that never looked at readonly at all.
===config===
suppress=MissingConstructor
===file===
<?php
class Counter {
    public readonly int $n;

    public function bump(): void {
        $this->n++;
//      ^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Counter::$n outside of constructor
    }
}
===expect===
