===description===
@psalm-self-out static<T> on a static:: call keeps the class's own inferred
type argument, instead of erasing it to a bare, unparameterized class.
===config===
suppress=UnusedParam,UnusedVariable
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

    public function useReset(): void {
        static::reset("hello");
        /** @mir-check $this is Box<string> */
        $_ = 1;
    }
}
===expect===
