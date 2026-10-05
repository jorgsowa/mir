===description===
class-string<T> over a class-level template is not resolved as a class in any method kind
===file===
<?php
/** @template T */
final class Holder {
    /** @param class-string<T> $c */
    public function call(string $c): void {
        $c::make();
        forward_static_call_array([$c, 'make'], []);
        echo $c::X;
        new $c();
        /** @mir-check $c is class-string<T> */
        $c;
    }

    /** @param class-string<T> $c */
    public static function viaStatic(string $c): void {
        $c::make();
    }

    /**
     * @template U
     * @param class-string<T> $c
     * @param class-string<U> $u
     */
    public function withOwnTemplate(string $c, string $u): void {
        $c::make();
        $u::make();
    }

    /** @param class-string<T> $c */
    public function viaClosure(string $c): \Closure {
        return fn() => $c::make();
    }
}

/** @template T */
trait HolderTrait {
    /** @param class-string<T> $c */
    public function call(string $c): void {
        $c::make();
    }
}
===expect===
