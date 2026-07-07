===description===
FP: an inferred binding that is still a partially-unresolved intersection
(e.g. `TChild&Countable`, where `TChild` is an enclosing, still-unbound
template) was treated as fully resolved because the "is this still
unresolved" check had no arm for `TIntersection`, so it was compared against
the bound literally instead of being skipped until a concrete call site
resolves it.
===config===
suppress=UnusedVariable,MissingReturnType,UnusedParam
===file===
<?php
interface Marker {}
class Foo implements Marker {}

/**
 * @template TChild
 */
class Builder {
    /**
     * @template U of Foo
     * @param U $c
     */
    public function accept($c): void {}

    /**
     * @param TChild&Marker $child
     */
    public function push($child): void {
        $this->accept($child);
    }
}
===expect===
