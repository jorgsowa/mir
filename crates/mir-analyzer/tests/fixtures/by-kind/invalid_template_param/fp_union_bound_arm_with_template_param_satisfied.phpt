===description===
FP: when a template bound is a union containing arms with template params
(e.g. T of string|list<I>|array<K, V>), an inferred type satisfying any arm
should pass the bound check — not trigger InvalidTemplateParam.
===file===
<?php

/**
 * @template I
 * @template K of array-key
 * @template V
 * @template T of string|list<I>|array<K, V>
 * @param T $value
 */
function accept(mixed $value): void {}

accept('hello');          // satisfies string arm
accept([1, 2, 3]);        // satisfies list<I> arm (I = int)
accept(['a' => 1]);       // satisfies array<K, V> arm
===expect===
UnusedParam@10:17: Parameter $value is never used
