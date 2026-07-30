===description===
Negative control for the elseif re-narrowing fix: a genuinely repeated
check within the same `&&` condition (the second operand narrowed by the
first, both written by hand rather than incidentally) must still be
flagged — the fix only stops the condition from being narrowed against
itself, not real redundancy within it.
===file===
<?php
class A {
    public array $values = [];
}
class B {
    public function __construct(public ?string $value) {}
}
function process(A|B $cell): void {
    if ($cell instanceof A) {
    } elseif ($cell->value !== null && $cell->value !== null) {
        echo $cell->value;
    }
}
===expect===
ImpossibleIdenticalComparison@10:39-10:60: '!==' between 'string' and 'null' is always true — these types can never be identical
