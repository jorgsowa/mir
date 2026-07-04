===description===
A 3-level @extends chain that re-templates at the middle class resolves to the concrete type
===file===
<?php
/** @template V */
class GrandParentBox {
    /**
     * @param V $v
     * @return V
     */
    public function process($v) {
        return $v;
    }
}

/**
 * @template U
 * @extends GrandParentBox<U>
 */
class ChildBox extends GrandParentBox {}

/** @extends ChildBox<int> */
class GrandChildBox extends ChildBox {}

$gc = new GrandChildBox();
$gc->process("this is a string, not an int");
===expect===
InvalidArgument@23:13-23:43: Argument $v of process() expects 'int', got '"this is a string, not an int"'
