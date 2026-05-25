===description===
nested conditional types with identical inner branches simplify recursively
===file===
<?php
/** @template T */
class Wrapper {}

class NestedFactory {
    /**
     * Nested conditional: outer and inner both have identical branches
     * @return ($x is null ? ($y is int ? Wrapper<string> : Wrapper<string>) : Wrapper<string>)
     */
    public function makeNested($x, $y): Wrapper { return new Wrapper(); }

    public Wrapper $wrapper;
}

$factory = new NestedFactory();
$result = $factory->makeNested(null, 1);
// Should resolve to Wrapper<string> (inner conditional collapses, then outer)
$factory->wrapper = $result;
===expect===
UnusedParam@10:32: Parameter $x is never used
UnusedParam@10:36: Parameter $y is never used
