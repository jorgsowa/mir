===description===
`$obj[$idx]` (`ArrayAccess::offsetGet()` fallback, no `@implements
ArrayAccess<TKey,TValue>` annotation) over a self-generic subclass that
reuses the same conventional template letter (`T`) as its
`@extends`-fixed ancestor resolves `offsetGet()`'s `@return T` using the
ANCESTOR's binding, not the subclass's own T — the ancestor declares
`offsetGet()`, not the subclass.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam,MissingPropertyType,MixedArrayOffset
===file===
<?php
/**
 * @template T
 */
class Box implements ArrayAccess {
    /** @var list<T> */
    private array $items = [];
    public function offsetExists(mixed $offset): bool {
        return isset($this->items[$offset]);
    }
    /** @return T */
    public function offsetGet(mixed $offset): mixed {
        return $this->items[$offset];
    }
    public function offsetSet(mixed $offset, mixed $value): void {}
    public function offsetUnset(mixed $offset): void {}
}

/**
 * @template T
 * @extends Box<int>
 */
class Wrapper extends Box {
    /** @var T */
    public $extra;
}

/** @param Wrapper<string> $w */
function collision(Wrapper $w): void {
    /** @mir-check $w[0] is int */
    $_ = $w[0];
}

// Cross-directional check: Wrapper's OWN member (`extra`, declared directly
// on Wrapper) must still resolve using Wrapper's own T.
/** @param Wrapper<string> $w */
function ownMemberStillCorrect(Wrapper $w): void {
    /** @mir-check $w->extra is string */
    $_ = $w->extra;
}
===expect===
