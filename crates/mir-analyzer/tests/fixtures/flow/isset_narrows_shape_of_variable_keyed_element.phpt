===description===
`isset($a[$i]['k'])` proves the optional key of the element's shape, for a variable or a property.
===file===
<?php
final class H {
    /** @var array<int, array{e?: Exception}> */
    private array $jobs = [];
    public function f(int $i): Exception {
        assert(isset($this->jobs[$i]['e']));
        return $this->jobs[$i]['e'];
    }
}
/** @param array<int, array{e?: Exception}> $jobs */
function g(array $jobs, int $i): Exception {
    if (isset($jobs[$i]['e'])) {
        return $jobs[$i]['e'];
    }
    return new Exception();
}
