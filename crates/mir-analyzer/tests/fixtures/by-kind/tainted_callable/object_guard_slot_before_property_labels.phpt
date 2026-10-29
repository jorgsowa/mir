===description===
A parameter the body proves an object, with `instanceof` or a callee's assertion, keeps its guard slot when the parameters and the properties the method reads would otherwise use every label slot.
===file===
<?php
class Query {
    public string $a = '';
    public string $b = '';
    public string $c = '';
    public string $d = '';
    public string $e = '';

    protected function isQueryable(mixed $value): bool {
        return $value instanceof self || $value instanceof Closure;
    }

    public function whereIn(mixed $column, mixed $values, string $boolean = 'and', bool $not = false): static {
//                          ^^^^^^^^^^^^^ UnusedParam: Parameter $column is never used
//                                                        ^^^^^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $boolean is never used
//                                                                                 ^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $not is never used
        if ($this->isQueryable($values)) {
            $values($this);
        }
        $this->a = $this->b . $this->c . $this->d . $this->e;
        return $this;
    }

    /** @return array{mixed, mixed} */
    public function prepare(mixed $value, mixed $operator): array {
        return [$value, $operator];
    }

    public function where(mixed $column, mixed $operator = null, mixed $value = null, string $boolean = 'and'): static {
//                        ^^^^^^^^^^^^^ UnusedParam: Parameter $column is never used
//                                                                                    ^^^^^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $boolean is never used
        [$value, $operator] = $this->prepare($value, $operator);
//       ^^^^^^ MixedAssignment: Variable $value is assigned a mixed type
//               ^^^^^^^^^ MixedAssignment: Variable $operator is assigned a mixed type
        if ($this->isQueryable($value)) {
            $value($this);
        }
        $this->a = $this->b . $this->c . $this->d . $this->e;
        return $this;
    }

    public function whereNot(mixed $column, mixed $values, string $boolean = 'and', bool $not = false): static {
//                           ^^^^^^^^^^^^^ UnusedParam: Parameter $column is never used
//                                                         ^^^^^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $boolean is never used
//                                                                                  ^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $not is never used
        if ($values instanceof Closure) {
            $values($this);
        }
        $this->a = $this->b . $this->c . $this->d . $this->e;
        return $this;
    }
}

function callers(Query $q): void {
    $token = (string) $_GET['t'];
//                    ^^^^^^^^^^ MixedArrayAccess: Array access on mixed type
    $q->whereIn('id', $token);
    $q->where('id', $token);
    $q->where('id', '=', $token);
    $q->whereNot('id', $token);
}
