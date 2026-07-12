===description===
does not report InvalidScope for this in free standing closure later bound via
Closure::bind. The residual MixedReturnStatement is expected: mir can't know
which class the closure will be bound to ahead of time, so `$this->value`
resolves to an unknown (mixed) type — an Info-level issue hidden from the
editor by default, not the InvalidScope false positive this fixture guards.
===file===
<?php
class Container {
    private int $value = 42;
}
$getter = function (): int {
    return $this->value;
};
$bound = Closure::bind($getter, new Container(), Container::class);
echo $bound();
===expect===
MixedReturnStatement@6:4-6:24: Cannot return a mixed type from function with declared return type 'int'
