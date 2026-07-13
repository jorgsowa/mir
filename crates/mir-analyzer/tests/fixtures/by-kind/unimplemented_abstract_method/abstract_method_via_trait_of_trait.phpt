===description===
FN: an abstract method declared only in a trait-of-a-trait (`trait Mid {
use Leaf; }`) was invisible to this check — the legacy ancestor walker's
trait branch didn't recurse into a trait's own transitively-used traits.
===file===
<?php
trait Leaf {
    abstract public function foo(): void;
}
trait Mid {
    use Leaf;
}
class Incomplete {
    use Mid;
}
===expect===
UnimplementedAbstractMethod@8:0-8:18: Class Incomplete must implement abstract method foo()
