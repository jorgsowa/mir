===description===
FP: conditional return (A is string ? X : Y) resolves to Y when a non-string is passed —
inactive if-true branch template vars must not leak into the result
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

/** @template-covariant T */
class Type {
    /** @return Type<int> */
    public static function int(): self { return new self; }

    /**
     * @template A
     * @template I
     * @param Type<A> $type
     * @psalm-return (
     *     A is string ? Type<list<I>> :
     *                   Type<int>
     * )
     */
    public function refined(self $type): self { return $type; }
}

// A = int → must pick else branch → Type<int>; no unbound I leak
$obj = new Type;
$result = $obj->refined(Type::int());
/** @mir-check $result is Type<int> */
echo "ok";
===expect===
