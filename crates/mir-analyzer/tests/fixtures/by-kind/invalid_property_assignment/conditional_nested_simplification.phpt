===description===
nested conditional types with identical inner branches simplify recursively
===file===
<?php
/** @template T */
class Wrapper {}

class NestedFactory {
//<^^^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class NestedFactory has uninitialized properties but no constructor
    /**
     * Nested conditional: outer and inner both have identical branches
     * @return ($x is null ? ($y is int ? Wrapper<string> : Wrapper<string>) : Wrapper<string>)
     */
    public function makeNested($x, $y): Wrapper { return new Wrapper(); }
//                             ^^ UnusedParam: Parameter $x is never used
//                                 ^^ UnusedParam: Parameter $y is never used

    public Wrapper $wrapper;
}

$factory = new NestedFactory();
$result = $factory->makeNested(null, 1);
/** @mir-check $result is Wrapper<string> */
$factory->wrapper = $result;
