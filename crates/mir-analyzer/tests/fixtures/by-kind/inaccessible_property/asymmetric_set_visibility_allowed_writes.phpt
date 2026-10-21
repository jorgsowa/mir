===description===
Writes inside the declaring class, in a subclass for `protected(set)`, and to ordinary properties are accessible.
===file===
<?php
class P {
    public private(set) int $a = 1;
    public protected(set) int $b = 2;
    public int $open = 0;
    public function set(): void {
        $this->a = 2;
        $this->b = 3;
    }
}

class Child extends P {
    public function f(): void {
        $this->b = 4;
    }
}

trait T {
    public private(set) int $t = 0;
    public function bumpT(): void {
        $this->t++;
    }
}

class UsesT {
    use T;
}

function outside(P $p, UsesT $u): int {
    $p->open = 1;
    $u->bumpT();
    return $p->a + $p->b + $u->t;
}
