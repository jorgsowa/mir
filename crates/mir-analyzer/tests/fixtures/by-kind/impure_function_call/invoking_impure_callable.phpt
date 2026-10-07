===description===
Invoking a callable that isn't known to be pure is impure in a @pure function;
pure-callable/pure-Closure values, closures written in the pure body and
first-class callables of pure functions are fine. `is_callable()` on a value
that may be a string autoloads, so it's impure too.
===file===
<?php
final class Invoke {
    /**
     * @pure
     * @param string|callable():string $message
     */
    public static function resolve(string|callable $message): string {
        return \is_callable($message) ? $message() : $message;
//             ^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function is_callable() in a @pure function
//                                      ^^^^^^^^^^ ImpureFunctionCall: Calling impure function $message() in a @pure function
    }

    /**
     * @pure
     * @param callable():string $f
     */
    public static function call(callable $f): string {
        return $f();
//             ^^^^ ImpureFunctionCall: Calling impure function $f() in a @pure function
    }

    /** @pure */
    public static function random(): int {
        $g = mt_rand(...);
        return $g();
//             ^^^^ ImpureFunctionCall: Calling impure function $g() in a @pure function
    }

    /**
     * @pure
     * @param list<int> $xs
     * @param callable(int):int $f
     * @return list<int>
     */
    public static function mapImpure(array $xs, callable $f): array {
        return array_map($f, $xs);
//             ^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function array_map() in a @pure function
    }

    /**
     * @pure
     * @param pure-callable():string $f
     */
    public static function callPure(callable $f): string {
        return $f();
    }

    /**
     * @pure
     * @param pure-Closure(int):int $f
     */
    public static function callPureClosure(Closure $f): int {
        return $f(1);
    }

    /** @pure */
    public static function local(): int {
        $g = fn(int $x): int => $x + 1;
        $h = function (): int {
            return 2;
        };
        $len = strlen(...);
        return $g(1) + $h() + $len('a');
    }

    /**
     * @pure
     * @param list<int> $xs
     * @param pure-callable(int):int $f
     * @return list<int>
     */
    public static function mapPure(array $xs, callable $f): array {
        return array_map($f, $xs);
    }

    public static function notPure(callable $f): mixed {
        return $f();
    }
}
