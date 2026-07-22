===description===
`$this->prop === []` / `!== []` (and loose `==`/`!=`) narrow a property
receiver to empty/non-empty the same way a plain variable already does.
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
final class Bag {
    /** @var array<int, string> */
    public array $items = [];

    public function strictNotEmpty(): void {
        if ($this->items !== []) {
            /** @mir-check $this->items is non-empty-array<int, string> */
            $_ = 1;
        }
    }

    public function strictEmpty(): void {
        if ($this->items === []) {
            /** @mir-check $this->items is array{} */
            $_ = 1;
        }
    }

    public function looseNotEmpty(): void {
        if ($this->items != []) {
            /** @mir-check $this->items is non-empty-array<int, string> */
            $_ = 1;
        }
    }

    public function reversedOperands(): void {
        if ([] !== $this->items) {
            /** @mir-check $this->items is non-empty-array<int, string> */
            $_ = 1;
        }
    }
}
===expect===
