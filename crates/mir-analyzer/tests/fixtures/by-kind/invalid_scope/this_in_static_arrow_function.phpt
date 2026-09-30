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
//                               ^^^^^ InvalidScope: $this cannot be used in a static method
//                               ^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
        return $f();;
//                  ^ UnreachableCode: Unreachable code detected
    }
}

===expect===
