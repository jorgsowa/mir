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
InvalidScope@5:33-5:38: $this cannot be used in a static method
UnreachableCode@6:20-6:21: Unreachable code detected
