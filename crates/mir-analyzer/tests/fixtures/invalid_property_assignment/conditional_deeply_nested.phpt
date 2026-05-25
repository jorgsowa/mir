===description===
deeply nested conditionals with multiple collapsible levels
===file===
<?php
class DeepFactory {
    /**
     * Three levels deep, all with identical branches
     * @return ($a is null ? ($b is int ? ($c is string ? Box<object> : Box<object>) : Box<object>) : Box<object>)
     */
    public function makeDeep($a, $b, $c): Box {}

    /** @var Box */
    public $box;
}

class Box<T> {}

$factory = new DeepFactory();
// All nested conditionals should collapse recursively to Box<object>
$result = $factory->makeDeep(null, 1, "x");
$factory->box = $result;
===expect===
