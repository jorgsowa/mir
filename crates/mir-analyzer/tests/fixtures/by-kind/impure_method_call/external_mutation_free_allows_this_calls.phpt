===description===
Calling non-pure methods on $this inside a @psalm-external-mutation-free method
is allowed — external-mutation-free only guards external objects.
===file===
<?php

class Counter {
    private int $n = 0;

    public function increment(): void {
        $this->n++;
    }

    /** @psalm-external-mutation-free */
    public function reset(): void {
        $this->increment();
    }
}
===expect===
