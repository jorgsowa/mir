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
//      ^^^^^^^^^^^^ InaccessibleProperty: Cannot access property P::$a
    }
}

function outside(P $p): int {
    $p->a = 5;
//  ^^^^^^^^^ InaccessibleProperty: Cannot access property P::$a
    $p->b = 6;
//  ^^^^^^^^^ InaccessibleProperty: Cannot access property P::$b
    $p->c = 7;
//  ^^^^^^^^^ InaccessibleProperty: Cannot access property P::$c
    $p->a++;
//  ^^^^^ InaccessibleProperty: Cannot access property P::$a
    $p->a += 2;
//  ^^^^^^^^^^ InaccessibleProperty: Cannot access property P::$a
    $p->list[] = 2;
//  ^^^^^^^^^^^^^^ InaccessibleProperty: Cannot access property P::$list
    return $p->a + $p->b + $p->c;
}
