===description===
FN: constructing a generic object from an argument whose type is itself an
enclosing, still-unbound class template (e.g. `new Box($child)` where
`$child : TChild`) discarded the whole `type_params` vec instead of binding
`T -> TChild`, because the "did this argument really bind something concrete"
check used `is_mixed()`, which treats an unconstrained template as mixed —
the same conflation `is_mixed_not_template()` exists to avoid elsewhere.
===config===
suppress=MissingReturnType,MissingPropertyType,UnusedParam,UnusedVariable
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    public $item;

    /** @param T $item */
    public function __construct($item) {
        $this->item = $item;
    }
}

/** @template TChild */
class Builder {
    /** @param TChild $child */
    public function push($child): void {
        $box = new Box($child);
        /** @mir-check $box is Box<TChild> */
        echo "ok";
    }
}
===expect===
