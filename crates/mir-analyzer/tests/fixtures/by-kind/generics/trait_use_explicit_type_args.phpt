===description===
`@use TraitName<T>` binds a used trait's own `@template` from an explicit
type-argument list, instead of always falling back to `mixed`.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingConstructor,UnusedParam
===file===
<?php
/** @template T */
trait Container {
    /** @var T */
    protected $value;
    /** @return T */
    public function get() { return $this->value; }
}

/** @use Container<int> */
class IntBox {
    use Container;
}

/** @use Container<string> */
class StringBox {
    use Container;
}

function readsIntBox(IntBox $box): void {
    /** @mir-check $box->get() is int */
    $_ = 1;
}

function readsStringBox(StringBox $box): void {
    /** @mir-check $box->get() is string */
    $_ = 1;
}
===expect===
