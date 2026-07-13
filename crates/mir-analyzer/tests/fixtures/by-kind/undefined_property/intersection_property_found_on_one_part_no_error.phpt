===description===
Sibling of intersection_property_missing_from_all_parts: a property found on one part is not flagged.
===file===
<?php
interface CountableIface {
    public function count(): int;
}
class Foo {
    public string $known = 'x';
}
function useIt(object $x): void {
    if ($x instanceof Foo && $x instanceof CountableIface) {
        echo $x->known;
    }
}
===expect===
