===description===
nested conditional types with identical inner branches simplify recursively
===file===
<?php
class NestedFactory {
    /**
     * Nested conditional: outer and inner both have identical branches
     * @return ($x is null ? ($y is int ? Wrapper<string> : Wrapper<string>) : Wrapper<string>)
     */
    public function makeNested($x, $y): Wrapper {}

    /** @var Wrapper */
    public $wrapper;
}

class Wrapper<T> {}

$factory = new NestedFactory();
$result = $factory->makeNested(null, 1);
// Should resolve to Wrapper<string> (inner conditional collapses, then outer)
$factory->wrapper = $result;
===expect===
