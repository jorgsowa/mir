===description===
`$x instanceof Subclass` on a receiver already known as `Ancestor<int>` must
narrow to a parameterized `Subclass<int>`, not a bare unparameterized
`Subclass` — narrow_instanceof_preserving_subtypes and its OR-chain sibling
narrow_or_instanceof_union built the narrowed atom with empty_type_params()
unconditionally, discarding the original atom's own type_params. That left
a later `Subclass` method's own `@return T` (or `@return TValue`) with
nothing to substitute, leaking the raw template atom into the caller
instead of the concrete bound type. Covers both an implicit passthrough
through a plain `extends` (same declared template arity, no explicit
`@extends` type args) and an explicit `@implements Iface<TKey, TValue>`
clause naming the subclass's own template params.
===config===
suppress=MissingPropertyType,MixedArrayOffset
===file===
<?php
/** @template T */
class Box {
    /** @param T $item */
    public function __construct(private $item) {}
    /** @return T */
    public function get() { return $this->item; }
}

/** @template T */
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

/**
 * @template TKey
 * @template TValue
 */
interface Mapish {
    /**
     * @param TKey $key
     * @return TValue
     */
    public function get($key);
}

/**
 * @template TKey
 * @template TValue
 * @implements Mapish<TKey, TValue>
 */
class ArrMap implements Mapish {
    /** @param array<TKey, TValue> $items */
    public function __construct(private array $items) {}

    /**
     * @param TKey $key
     * @return TValue
     */
    public function get($key) {
        return $this->items[$key];
    }
}

/** @param Mapish<string, int> $m */
function useIt($m): int {
    if ($m instanceof ArrMap) {
        $v = $m->get('x');
        /** @mir-check $v is int */
        return $v;
    }
    return 0;
}
===expect===
