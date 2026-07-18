===description===
`$this->prop < N` / `N < $this->prop` (and `<=`/`>`/`>=`) narrow a property
receiver's integer range the same way a plain variable already does.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
final class Counter {
    /** @var int<0, 10> */
    public int $count = 0;

    public function literalOnRight(): void {
        if ($this->count > 0) {
            /** @mir-check $this->count is int<1, 10> */
            $_ = 1;
        }
    }

    public function literalOnLeft(): void {
        if (0 < $this->count) {
            /** @mir-check $this->count is int<1, 10> */
            $_ = 1;
        }
    }

    public function falseBranchUnaffected(): void {
        if ($this->count > 0) {
        } else {
            /** @mir-check $this->count is int<0, 0> */
            $_ = 1;
        }
    }
}
===expect===
