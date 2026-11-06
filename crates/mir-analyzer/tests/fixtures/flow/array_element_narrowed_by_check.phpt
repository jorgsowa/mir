===description===
A check on `$a[$k]`, `$a['x']` or `$this->m[$k]` narrows that element of a generic array until the array or key changes.
===file===
<?php
class E extends Exception {}
class J {
    /** @var array<string, array{int}|E> */
    private array $m = [];

    /**
     * @param array<string, int|string|null> $a
     * @throws E
     */
    public function f(string $k, array $a): void {
        if ($this->m[$k] instanceof E) {
            throw $this->m[$k];
        }
        if (is_string($a[$k])) {
            $s = $a[$k];
            /** @mir-check $s is string */
            echo $s;
        }
        if ($a['x'] !== null) {
            $x = $a['x'];
            /** @mir-check $x is int|string */
            echo $x;
        }
    }

    /** @param array<string, int|string|E> $a */
    public function negated(string $k, array $a): void {
        if (is_string($a[$k]) || $a[$k] instanceof E) {
            $v = $a[$k] instanceof E ? 'e' : $a[$k];
            /** @mir-check $v is string */
            echo $v;
        }
    }

    /** @param array<string, int|string> $a */
    public function stale(string $k, array $a): void {
        if (is_string($a[$k])) {
            $k = 'other';
            $s = $a[$k];
            /** @mir-check $s is int|string */
            echo $s;
        }
        if (is_string($a[$k])) {
            $a[$k] = 1;
            $s = $a[$k];
            /** @mir-check $s is int|string */
            echo $s;
        }
        if ($this->m[$k] instanceof E) {
            $this->m[$k] = [1];
            $e = $this->m[$k];
            /** @mir-check $e is array{0: int}|E */
            echo $e instanceof E;
        }
    }
}
