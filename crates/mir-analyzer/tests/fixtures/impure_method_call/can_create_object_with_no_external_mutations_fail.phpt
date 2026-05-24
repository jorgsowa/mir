===description===
canCreateObjectWithNoExternalMutationsFail
===file===
<?php
class Counter {
    private int $count = 0;

    /** @psalm-mutation-free */
    public function __construct(int $count) {
        $this->count = $count;
    }

    public function increment() : void {
        $this->count += rand(0, 5);
    }
}

/** @psalm-pure */
function makesACounter(int $i) : Counter {
    $c = new Counter($i);
    $c->increment();
    return $c;
}
===expect===
ImpureMethodCall
===ignore===
TODO
