===description===
Open shapes grown in branches and unioned with a bare `array` are accepted by
a shaped property, like a bare `array` alone (a coercion, not an invalid
assignment), whether or not they carry the property's keys. A union with an
atom that is not an array is still rejected.
===file===
<?php
final class Box {
    /** @var array{a: int, b: string} */
    private array $shaped = [];

    private function load(): array { return []; }

    public function bare(): void {
        $this->shaped = $this->load();
    }

    public function grown_in_branch(bool $c): void {
        $r = $this->load();
        if ($c) {
            $r['b'] = 'x';
            $r['a'] = 1;
        }
        $this->shaped = $r;
    }

    public function grown_in_both_branches(bool $c): void {
        $r = $this->load();
        if ($c) {
            $r['b'] = 'x';
            $r['a'] = 1;
        } else {
            $r['b'] = 'y';
            $r['a'] = 2;
        }
        /** @mir-check $r is array{'b': "x", 'a': 1}|array{'b': "y", 'a': 2} */
        $this->shaped = $r;
    }

    public function partial_shapes_in_branches(bool $c, bool $d): void {
        $r = $this->load();
        if ($c) {
            $r['a'] = 1;
        }
        if ($d) {
            $r['extra'] = 'x';
        }
        $this->shaped = $r;
    }

    public function scalar_in_union(bool $c): void {
        $r = $this->load();
        if ($c) {
            $r = 1;
        }
        $this->shaped = $r;
    }
}
===expect===
PropertyTypeCoercion@9:8-9:37: Property $shaped expects 'array{'a': int, 'b': string}', cannot assign 'array' — coercion may fail at runtime
PropertyTypeCoercion@18:8-18:26: Property $shaped expects 'array{'a': int, 'b': string}', cannot assign 'array{'b': "x", 'a': 1}|array' — coercion may fail at runtime
PropertyTypeCoercion@42:8-42:26: Property $shaped expects 'array{'a': int, 'b': string}', cannot assign 'array{'a': 1, 'extra': "x"}|array{'extra': "x"}|array{'a': 1}|array' — coercion may fail at runtime
InvalidPropertyAssignment@50:8-50:26: Property $shaped expects 'array{'a': int, 'b': string}', cannot assign '1|array'
