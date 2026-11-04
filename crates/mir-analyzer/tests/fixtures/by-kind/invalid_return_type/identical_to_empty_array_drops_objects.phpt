===description===
`$x === []` and `$this->p == []` rule out object types
===file===
<?php
final class D {
    private D|array $p = [];

    public function local(D|array $x): array {
        if ([] === $x) {
            return $x;
        }
        return [];
    }

    public function prop(): array {
        if ($this->p == []) {
            return $this->p;
        }
        return [];
    }
}
