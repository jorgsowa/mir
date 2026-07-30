===description===
This in static arrow function (D4: also fires MixedReturnStatement now,
same as the equivalent regular static closure — $this is unresolvable
in a static scope, so the property access on it is only ever mixed)
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
MixedReturnStatement@5:33-5:41: Cannot return a mixed type from function with declared return type 'int'
UnreachableCode@6:20-6:21: Unreachable code detected
