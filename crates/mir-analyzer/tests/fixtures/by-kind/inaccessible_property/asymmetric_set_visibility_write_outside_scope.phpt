===description===
A write to a `private(set)` or `protected(set)` property outside its scope is inaccessible; reads stay public.
===file===
<?php
class P {
    public private(set) int $a = 1;
    public protected(set) int $b = 2;
    public private(set) array $list = [];
    public function __construct(public private(set) int $c = 3) {}
    public function bump(): void {
        $this->a = 5;
        $this->a++;
        $this->b += 1;
        $this->c = 4;
        $this->list[] = 1;
    }
}

class Child extends P {
    public function f(): void {
        $this->b = 6;
        $this->a = 7;
    }
}

function outside(P $p): int {
    $p->a = 5;
    $p->b = 6;
    $p->c = 7;
    $p->a++;
    $p->a += 2;
    $p->list[] = 2;
    return $p->a + $p->b + $p->c;
}
===expect===
InaccessibleProperty@19:8-19:20: Cannot access property P::$a
InaccessibleProperty@24:4-24:13: Cannot access property P::$a
InaccessibleProperty@25:4-25:13: Cannot access property P::$b
InaccessibleProperty@26:4-26:13: Cannot access property P::$c
InaccessibleProperty@27:4-27:9: Cannot access property P::$a
InaccessibleProperty@28:4-28:14: Cannot access property P::$a
InaccessibleProperty@29:4-29:18: Cannot access property P::$list
