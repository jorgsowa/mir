===description===
`@psalm-self-out static<T>` on a method reached through `static::` must both
(a) resolve `static` to the actual late-bound receiver class (a subclass),
not the declaring class, and (b) substitute `T` with this call's inferred
argument type — combining the two behaviors the `self`/`static`-with-generics
sentinel has to get right at once.
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
class Box {
    /** @param T $value */
    public function __construct($value) {}

    /**
     * @param T $value
     * @psalm-self-out static<T>
     */
    public static function reset($value): void {}
}

class SubBox extends Box {
    public function useReset(): void {
        static::reset("hello");
        /** @mir-check $this is SubBox<string> */
        $_ = 1;
    }
}
===expect===
