===description===
FP: conditional return (A is list ? X : Y) resolves to X when a list is passed —
inactive else-branch template vars must not leak
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

/** @template-covariant T */
class Type {
    /**
     * @template I
     * @return Type<list<I>>
     */
    public static function listOf(): self { return new self; }

    /**
     * @template A
     * @template I
     * @template K of array-key
     * @template V
     * @param Type<A> $type
     * @psalm-return (
     *     A is list         ? Type<list<I>>     :
     *                         Type<array<K, V>>
     * )
     */
    public function refined(self $type): self { return $type; }

    /**
     * @template I
     * @param Type<list<I>> $item
     * @return Type<list<I>>
     */
    public static function wrap(self $item): self { return new self; }
}

// A = list<I> → must pick first branch → Type<list<I>>; no unbound K/V leak
$t = Type::wrap((new Type)->refined(Type::listOf()));
===expect===
