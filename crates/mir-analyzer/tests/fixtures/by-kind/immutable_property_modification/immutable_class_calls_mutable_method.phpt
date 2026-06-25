===description===
Calling a non-mutation-free instance method on $this inside a @psalm-immutable
class emits ImpureMethodCall, because the callee may mutate object state.
===file===
<?php

/** @psalm-immutable */
class Point {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    public function reset(): void {
        $this->doReset();
    }

    private function doReset(): void {
        $this->x = 0.0;
        $this->y = 0.0;
    }
}
===expect===
ImpureMethodCall@11:8-11:24: Calling impure method doReset() in a pure or immutable context
ImmutablePropertyModification@15:8-15:22: Assigning to property x of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
ImmutablePropertyModification@16:8-16:22: Assigning to property y of $this in an immutable context (@psalm-immutable class or @psalm-mutation-free method)
