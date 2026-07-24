===description===
@psalm-self-out on a method reached through an intersection-typed receiver
(`A&B`) retypes the whole intersection atomic, not silently dropped.
===config===
suppress=UnusedParam
===file===
<?php
interface HasName {}
interface Fluent {
    /** @psalm-self-out Named */
    public function nameIt(string $n): static;
}
class Named implements HasName, Fluent {
    public function nameIt(string $n): static { return $this; }
}

/** @param Fluent&HasName $x */
function test($x): void {
    $x->nameIt('a');
    /** @mir-check $x is Named */
    $_ = 1;
}
===expect===
