===description===
does not report InvalidScope for $this in a free standing arrow function later
bound via Closure::bindTo (D4: MixedReturnStatement now fires though, same as
the equivalent regular closure — $this's real class is only known at bindTo
time, so the property access is genuinely unresolvable to more than mixed).
===file===
<?php
class Container {
    private int $value = 42;
}
$getter = fn (): int => $this->value;
$bound = $getter->bindTo(new Container(), Container::class);
echo $bound();
===expect===
MixedReturnStatement@5:24-5:36: Cannot return a mixed type from function with declared return type 'int'
