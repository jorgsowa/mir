===description===
deeply nested conditionals with multiple collapsible levels
===file===
<?php
/** @template T */
class Box {}

class DeepFactory {
//<^^^^^^^^^^^^^^^^^^^ MissingConstructor: Class DeepFactory has uninitialized properties but no constructor
    /**
     * Three levels deep, all with identical branches
     * @return ($a is null ? ($b is int ? ($c is string ? Box<object> : Box<object>) : Box<object>) : Box<object>)
     */
    public function makeDeep($a, $b, $c): Box { return new Box(); }
//                           ^^ UnusedParam: Parameter $a is never used
//                               ^^ UnusedParam: Parameter $b is never used
//                                   ^^ UnusedParam: Parameter $c is never used

    public Box $box;
}

$factory = new DeepFactory();
$result = $factory->makeDeep(null, 1, "x");
/** @mir-check $result is Box<object> */
$factory->box = $result;
===expect===
