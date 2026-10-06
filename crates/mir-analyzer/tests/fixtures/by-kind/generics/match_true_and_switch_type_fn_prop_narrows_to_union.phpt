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
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class HasScalarProp {
//<^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class HasScalarProp has uninitialized properties but no constructor
    /** @var int|string|bool */
    public mixed $x;

    private function acceptIntOrString(int|string $v): void {}
//                                     ^^^^^^^^^^^^^ UnusedParam: Parameter $v is never used

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
