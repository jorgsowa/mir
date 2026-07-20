===description===
`foreach` over a self-generic subclass that reuses the same conventional
template letter (`T`) as its `@extends`-fixed ancestor resolves
`current()`'s `@return T` using the ANCESTOR's binding (Box's own T,
fixed via `@extends Box<int>`), not the subclass's own T — the ancestor
declares `current()`, not the subclass.
===config===
suppress=UnusedVariable,MissingConstructor,UnusedParam,MissingPropertyType
===file===
<?php
/**
 * @template T
 */
class Box implements Iterator {
    /** @var list<T> */
    private array $items = [];
    private int $pos = 0;
    public function rewind(): void {
        $this->pos = 0;
    }
    public function valid(): bool {
        return isset($this->items[$this->pos]);
    }
    /** @return T */
    public function current(): mixed {
        return $this->items[$this->pos];
    }
    public function key(): int {
        return $this->pos;
    }
    public function next(): void {
        $this->pos++;
    }
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
    foreach ($w as $v) {
        /** @mir-check $v is int */
        $_ = $v;
    }
}

// Cross-directional check: Wrapper's OWN member (`extra`, declared directly
// on Wrapper) must still resolve using Wrapper's own T — the fix must not
// overcorrect and let the ancestor's binding leak into the subclass's own
// declarations.
/** @param Wrapper<string> $w */
function ownMemberStillCorrect(Wrapper $w): void {
    /** @mir-check $w->extra is string */
    $_ = $w->extra;
}
===expect===
