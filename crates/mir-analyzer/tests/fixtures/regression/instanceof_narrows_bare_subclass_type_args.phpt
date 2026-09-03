===description===
`$x instanceof Subclass` where `Subclass` is totally bare (no own
`@template` at all, e.g. `class IntBox extends Box {}`) must still project
the receiver's known type args onto it via the arity-match passthrough —
project_type_params_onto_subclass looked up the subclass's own-only
template params to decide arity, which is always empty for a bare
subclass, so the passthrough branch was unreachable and the ancestor's
type args were silently dropped.
===config===
suppress=MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @param T $item */
    public function __construct(private $item) {}
    /** @return T */
    public function get() { return $this->item; }
}

class IntBox extends Box {}

/** @param Box<int> $b */
function unwrapIfIntBox(Box $b): int {
    if ($b instanceof IntBox) {
        $v = $b->get();
        /** @mir-check $v is int */
        return $v;
    }
    return 0;
}
===expect===
