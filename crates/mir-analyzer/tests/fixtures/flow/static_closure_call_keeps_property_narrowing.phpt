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
    }

    public function staticReceivesThis(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = static fn ($o) => $o;
            $f($this);
        }
        return $this->bc;
    }

    /** @param \Closure(\stdClass): \stdClass $f */
    public function closureParam(\Closure $f): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f($this->bc);
        }
        return $this->bc;
    }

    public function methodCallable(): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = $this->reset(...);
            $f();
        }
        return $this->bc;
    }

    public function staticOrNot(bool $flag): \stdClass {
        if ($this->bc === null) {
            $this->bc = new \stdClass();
            $f = $flag ? static fn ($o) => $o : fn ($o) => $o;
            $f($this->bc);
        }
        return $this->bc;
    }

    private function reset(): void {
        $this->bc = null;
    }
}
===expect===
NullableReturnStatement@30:8-30:25: Return type 'stdClass|null' is not compatible with declared 'stdClass'
NullableReturnStatement@39:8-39:25: Return type 'stdClass|null' is not compatible with declared 'stdClass'
NullableReturnStatement@48:8-48:25: Return type 'stdClass|null' is not compatible with declared 'stdClass'
NullableReturnStatement@57:8-57:25: Return type 'stdClass|null' is not compatible with declared 'stdClass'
NullableReturnStatement@66:8-66:25: Return type 'stdClass|null' is not compatible with declared 'stdClass'
