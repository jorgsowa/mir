===description===
Comma-separated `match(true)` arm conditions and `switch(true)` fallthrough
labels over scalar type-check functions on a property receiver
(`is_int($this->x), is_string($this->x)`) are OR semantics — the arm/case
must narrow $this->x to int|string, not collapse to just the last disjunct
via sequential (AND) narrowing, mirroring the existing plain-variable
behavior. The match arm passes the narrowed property straight to a
strictly-typed `int|string` parameter (rather than an `@mir-check` inside a
closure) since a nested closure doesn't inherit property-refinement state.
===config===
suppress=UnusedVariable
===file===
<?php
class HasScalarProp {
    /** @var int|string|bool */
    public mixed $x;

    private function acceptIntOrString(int|string $v): void {}

    public function matchArm(): void {
        match (true) {
            is_int($this->x), is_string($this->x) => $this->acceptIntOrString($this->x),
            default => null,
        };
    }

    public function switchFallthrough(): void {
        switch (true) {
            case is_int($this->x):
            case is_string($this->x):
                /** @mir-check $this->x is int|string */
                $_ = 1;
                break;
        }
    }
}
===expect===
MissingConstructor@2:0-2:21: Class HasScalarProp has uninitialized properties but no constructor
UnusedParam@6:39-6:52: Parameter $v is never used
