===description===
Callback-taking builtins are pure only when their callback is: a closure
(checked in the pure scope itself), null, or a callable string/first-class
callable naming a pure function or method.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function record(string $value): string {
    echo $value;
    return $value;
}

final class Formatter {
    /** @pure */
    public static function label(int $value): string {
        return 'v' . $value;
    }

    /** @pure */
    public static function isPositive(int $value): bool {
        return $value > 0;
    }

    public static function logged(int $value): string {
        echo $value;
        return (string) $value;
    }

    /**
     * @pure
     * @param list<int> $values
     */
    public static function pureCallbacks(array $values): string {
        $labels = array_map(static::label(...), $values);
        /** @mir-check $labels is list<string> */
        $upper = array_map(strtoupper(...), $labels);
        $trimmed = array_map('trim', $upper);
        $doubled = array_map(fn (int $v): int => $v * 2, $values);
        $positive = array_filter($doubled, function (int $v): bool { return $v > 0; });
        $nonEmpty = array_filter($trimmed);
        $zipped = array_map(null, $values, $values);
        usort($values, static fn (int $a, int $b): int => $a <=> $b);
        $sum = array_reduce($values, fn (int $c, int $v): int => $c + $v, 0);
        $any = array_any(callback: self::isPositive(...), array: $values);
        return implode(',', $nonEmpty) . count($positive) . count($zipped) . $sum . (int) $any;
    }

    /**
     * @pure
     * @param list<int> $values
     * @param callable(int): string $callback
     * @param list<mixed> $args
     */
    public static function impureCallbacks(array $values, callable $callback, array $args): void {
        array_map(self::logged(...), $values);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function array_map() in a @pure function
        array_map(record(...), ['a']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function array_map() in a @pure function
        array_map('error_log', ['a']);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function array_map() in a @pure function
        array_map($callback, $values);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function array_map() in a @pure function
        array_map(...$args);
//      ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function array_map() in a @pure function
        array_map(fn (int $v): string => record((string) $v), $values);
//                                       ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function record() in a @pure function
    }
}
