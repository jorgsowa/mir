===description===
False-positive guards for the readonly double-init check: a write on only
ONE branch of an if/else isn't a DEFINITE prior initialization (only one
runtime path actually wrote it), so a write after the merge is the first
write on that path, not a re-init. A single write inside a loop body is
likewise not flagged — the loop might run zero times, so the merge with the
"didn't run" path clears the definite-init mark (mir does not yet model
"this loop might run more than once", a known limitation). Writes to
different receiver instances of the same class must not cross-contaminate.
===config===
suppress=UnusedParam,MissingConstructor,MixedAssignment
===file===
<?php
class Box {
    public readonly int $value;

    public function conditionalThenUnconditional(bool $cond, int $v): void {
        if ($cond) {
            $this->value = $v;
        }
        // Not a definite double-init: the if-branch might not have run.
        $this->value = $v;
    }

    public function writtenOnceInLoop(array $items): void {
        foreach ($items as $item) {
            $this->value = $item;
        }
    }

    public function differentInstances(Box $other, int $v): void {
        $this->value = $v;
        $other->value = $v;
    }
}
===expect===
