===description===
FP: conditional return type (A is string ? X : A is list ? Y : Z) should resolve to X
when the call-site argument is string — leaking unbound template vars I/K/V from the
inactive branches must not trigger InvalidTemplateParam.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

/** @template-covariant T */
class Type {
    /** @return Type<string> */
    public static function string(): self { return new self; }

    /**
     * @template A
     * @template I
     * @template K
     * @template V
     * @param Type<A> $type
     * @psalm-return (
     *     A is string ? Type<string>      :
     *     A is list   ? Type<list<I>>     :
     *                   Type<array<K, V>>
     * )
     */
    public function refined(self $type): self { return $type; }

    /**
     * @template I
     * @param Type<I> $item
     * @return Type<list<I>>
     */
    public static function list(self $item): self { return new self; }
}

// A = string → must pick first branch → Type<string>; no unbound I/K/V leak
$t = Type::list((new Type)->refined(Type::string()));
===expect===
