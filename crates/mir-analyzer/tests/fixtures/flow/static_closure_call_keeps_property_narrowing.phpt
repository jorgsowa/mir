===description===
Calling a `static` closure value keeps property narrowing: it has no `$this` to
mutate. Any other callable (non-static closure, `Closure` param, first-class
callable of a method) or `$this` passed as an argument still clears it.
===file===
<?php
final class Holder {
    private ?\stdClass $bc = null;

    public function staticArrow(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = static fn ($o) => $o;
            $f($this->bc);
        }
        /** @mir-check $this->bc is stdClass */
        return $this->bc;
    }

    public function staticFunction(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = static function (\stdClass $o): \stdClass { return $o; };
            $f($this->bc);
        }
        return $this->bc;
    }

    public function nonStatic(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = fn ($o) => $o;
            $f($this->bc);
        }
        return $this->bc;
//      ^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'stdClass|null' is not compatible with declared 'stdClass'
    }

    public function staticReceivesThis(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = static fn ($o) => $o;
            $f($this);
        }
        return $this->bc;
//      ^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'stdClass|null' is not compatible with declared 'stdClass'
    }

    /** @param \Closure(\stdClass): \stdClass $f */
    public function closureParam(\Closure $f): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f($this->bc);
        }
        return $this->bc;
//      ^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'stdClass|null' is not compatible with declared 'stdClass'
    }

    public function methodCallable(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = $this->reset(...);
            $f();
        }
        return $this->bc;
//      ^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'stdClass|null' is not compatible with declared 'stdClass'
    }

    public function reboundStatic(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            (static fn (\stdClass $o): \stdClass => $o)->bindTo(null, self::class)($this->bc);
        }
        /** @mir-check $this->bc is stdClass */
        return $this->bc;
    }

    public function reboundNonStatic(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            (fn (\stdClass $o): \stdClass => $o)->bindTo($this)($this->bc);
        }
        return $this->bc;
//      ^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'stdClass|null' is not compatible with declared 'stdClass'
    }

    public function staticOrNot(bool $flag): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = $flag ? static fn ($o) => $o : fn ($o) => $o;
            $f($this->bc);
        }
        return $this->bc;
//      ^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'stdClass|null' is not compatible with declared 'stdClass'
    }

    private function reset(): void {
        $this->bc = null;
    }
}
