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
/** @mir-check $result is Wrapper<string> */
$factory->wrapper = $result;
===expect===
MissingConstructor@5:0-5:21: Class NestedFactory has uninitialized properties but no constructor
UnusedParam@10:32-10:34: Parameter $x is never used
UnusedParam@10:36-10:38: Parameter $y is never used
