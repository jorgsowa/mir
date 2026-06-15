===description===
UnnecessaryVarAnnotation fires when a @var annotation exactly matches the
inferred type of a simple assignment. Narrowing, widening (e.g. a literal to
its base type) and mixed-typed RHS stay silent.
===file===
<?php
function get(): string { return 'x'; }
/** @return string|null */
function maybe() { return null; }

function f(): void {
    /** @var string $a */
    $a = get();

    /** @var string $b */
    $b = maybe();

    /** @var int $c */
    $c = 1;

    echo $a, $b, $c;
}
===expect===
UnnecessaryVarAnnotation@8:4-8:15: @var annotation for $a is unnecessary
