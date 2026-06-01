===description===
static method nested conditional return type must be widened at call site — no InvalidArgument (regression guard)
===config===
suppress=UnusedParam,UnusedVariable,UnusedFunction,UnusedClass
===file===
<?php

class Arr {
    /**
     * @template TKey of array-key
     * @template TValue
     * @param array<TKey, TValue>|TValue|null $value
     * @return ($value is null ? array{} : ($value is array<TKey, TValue> ? array<TKey, TValue> : array{TValue}))
     */
    public static function wrap(mixed $value): array {
        if (is_null($value)) return [];
        return is_array($value) ? $value : [$value];
    }
}

/** @param array<mixed> $a */
function takesArray(array $a): void {}

// Each concrete arg should resolve the conditional fully — no TConditional at call site.
takesArray(Arr::wrap(null));
takesArray(Arr::wrap('hello'));
takesArray(Arr::wrap(['a', 'b']));

// Union arg: conditional widens to both branches — still a subtype of array.
/** @var string|null $x */
$x = rand() ? 'foo' : null;
takesArray(Arr::wrap($x));
===expect===
