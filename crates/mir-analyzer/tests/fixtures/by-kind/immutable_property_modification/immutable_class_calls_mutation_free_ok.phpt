===description===
Calling a @psalm-mutation-free method on $this from within a @psalm-immutable
class is allowed — the callee is guaranteed not to mutate instance state.
===file===
<?php

/** @psalm-immutable */
class Vector {
    public function __construct(
        public float $x,
        public float $y,
    ) {}

    /** @psalm-mutation-free */
    public function length(): float {
        return sqrt($this->x ** 2 + $this->y ** 2);
    }

    public function normalize(): string {
        $len = $this->length();
        return "({$this->x}/{$len}, {$this->y}/{$len})";
    }
}
===expect===
