===description===
Same fix as the instance-method case, for the static-call path: a static
factory's `@param-out T` must substitute the class template inferred from
its own arguments (the same binding `@return static` already resolves
through) before writing back to the caller's variable.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @param T $seed */
    private function __construct(private $seed) {}

    /**
     * @param T $seed
     * @param-out T $out
     * @return static
     */
    public static function makeAndFill($seed, mixed &$out): static {
        $out = $seed;
        return new static($seed);
    }
}

Box::makeAndFill(42, $result);
/** @mir-check $result is int */
$_ = $result;
===expect===
