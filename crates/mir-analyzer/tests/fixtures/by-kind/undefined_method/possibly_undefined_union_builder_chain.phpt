===description===
Union receiver from a parent-returning `end()` where only one member declares
the method: the lacking members are possibly-undefined, never an Error.
===file===
<?php
interface ParentNode {}
class Builder implements ParentNode {
    public function leaf(): Leaf { return new Leaf(); }
}
class Branch {
    public function children(): Builder { return new Builder(); }
}
class Leaf {
    /** @return ParentNode|Builder|Branch|null */
    public function end() { return null; }
}
function test(Leaf $leaf): void {
    $leaf->end()->leaf();
//  ^^^^^^^^^^^^^^^^^^^^ PossiblyUndefinedMethod: Method Branch::leaf() might not exist
//  ^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method leaf() on possibly null value
}
