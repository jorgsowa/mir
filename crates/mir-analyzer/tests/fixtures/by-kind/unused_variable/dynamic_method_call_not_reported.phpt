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
UnusedParam@3:29-3:40: Parameter $col is never used
UnusedParam@3:42-3:53: Parameter $vals is never used
UnusedParam@4:35-4:46: Parameter $col is never used
UnusedParam@4:48-4:59: Parameter $vals is never used
UnusedParam@6:49-6:60: Parameter $key is never used
