===description===
variable used as dynamic method name is not reported
===file===
<?php
class EloquentBuilder {
    public function whereIn(string $col, array $vals): static { return $this; }
//                          ^^^^^^^^^^^ UnusedParam: Parameter $col is never used
//                                       ^^^^^^^^^^^ UnusedParam: Parameter $vals is never used
    public function whereInStrict(string $col, array $vals): static { return $this; }
//                                ^^^^^^^^^^^ UnusedParam: Parameter $col is never used
//                                             ^^^^^^^^^^^ UnusedParam: Parameter $vals is never used

    protected function loadMorphTo(bool $isInt, string $key): void {
        $whereIn = $isInt ? 'whereIn' : 'whereInStrict';
        $this->$whereIn($key, []);
    }
}
