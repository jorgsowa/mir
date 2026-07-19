===description===
`project_type_params_onto_subclass`'s implicit-passthrough arity check
compared the subclass's ancestor-resolved arity against the narrowed
atom's OWN-declared arity — wrong when the atom itself is a bare re-extend
hop with no own @template. `MediumBox` (bare `extends Box`) has 0 own
template params but an ancestor-resolved arity of 1; comparing against
that 0 always failed the arity match, so `$b instanceof SmallBox` on a
`MediumBox<int>` receiver dropped the type args entirely instead of
projecting `SmallBox<int>`.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @var T */
    private $value;

    /** @return T */
    public function get()
    {
        return $this->value;
    }
}

class MediumBox extends Box {}

class SmallBox extends MediumBox {}

/** @param MediumBox<int> $b */
function narrowsThroughBareChain(Box $b): void {
    if ($b instanceof SmallBox) {
        /** @mir-check $b is SmallBox<int> */
        $item = $b->get();
        /** @mir-check $item is int */
        $_ = 1;
    }
}
===expect===
