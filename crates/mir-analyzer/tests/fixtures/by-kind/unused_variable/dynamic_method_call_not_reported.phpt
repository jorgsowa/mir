===description===
variable used as dynamic method name is not reported
===file===
<?php
class EloquentBuilder {
    public function whereIn(string $col, array $vals): static { return $this; }
    public function whereInStrict(string $col, array $vals): static { return $this; }

    protected function loadMorphTo(bool $isInt, string $key): void {
        $whereIn = $isInt ? 'whereIn' : 'whereInStrict';
        $this->$whereIn($key, []);
    }
}
===expect===
UnusedParam@3:28-3:39: Parameter $col is never used
UnusedParam@3:41-3:52: Parameter $vals is never used
UnusedParam@4:34-4:45: Parameter $col is never used
UnusedParam@4:47-4:58: Parameter $vals is never used
UnusedParam@6:48-6:59: Parameter $key is never used
