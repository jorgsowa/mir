===description===
This in static arrow function
===file===
<?php
class C {
    public int $a = 1;
    public function f(): int {
        $f = static fn(): int => $this->a;
        return $f();;
    }
}

===expect===
InvalidScope@5:34-5:39: $this cannot be used in a static method
UnreachableCode@6:21-6:22: Unreachable code detected
