===description===
`@param-out T` on a generic class's method must substitute the receiver's
own bound type param before writing back to the caller's variable, the same
way a normal `@param T`/`@return T` already does.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/** @template T */
class Box {
    /** @param T $seed */
    public function __construct(private $seed) {}

    /**
     * @param-out T $out
     */
    public function fill(mixed &$out): void {
        $out = $this->seed;
    }
}

$box = new Box(42);
$box->fill($result);
/** @mir-check $result is int */
$_ = $result;
===expect===
