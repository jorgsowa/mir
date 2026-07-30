===description===
An `elseif` condition with an internal `&&` must not be re-checked against
a context where its own earlier operand's narrowing already applied — that
turned a genuine `!== null` check into a false "always true" tautology.
`else { if (...) }` (not `elseif`) doesn't hit this path at all.
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
    } elseif ($cell instanceof B && $cell->value !== null) {
        echo $cell->value;
    }
}
===expect===
