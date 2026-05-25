===description===
deeply nested conditionals with multiple collapsible levels
===file===
<?php
/** @template T */
class Box {}

class DeepFactory {
    /**
     * Three levels deep, all with identical branches
     * @return ($a is null ? ($b is int ? ($c is string ? Box<object> : Box<object>) : Box<object>) : Box<object>)
     */
    public function makeDeep($a, $b, $c): Box { return new Box(); }

    public Box $box;
}

$factory = new DeepFactory();
$result = $factory->makeDeep(null, 1, "x");
/** @mir-check $result is Box<object> */
$factory->box = $result;
===expect===
UnusedParam@10:30: Parameter $a is never used
UnusedParam@10:34: Parameter $b is never used
UnusedParam@10:38: Parameter $c is never used
