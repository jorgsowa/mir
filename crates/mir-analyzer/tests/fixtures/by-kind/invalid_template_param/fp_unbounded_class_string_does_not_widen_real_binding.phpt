===description===
An unbounded class-string next to a concrete binding does not widen it; a concrete violating class-string still flags.
===file===
<?php
interface Named {}
final class Good implements Named {}
final class Bad {}

/**
 * @template T of Named
 * @param class-string<T> $a
 * @param class-string<T> $b
 * @return T
 */
function pair(string $a, string $b): object { return new $a(new $b()); }

/**
 * @template T of Named
 * @param class-string<T> $class
 */
function register(string $class): void { echo $class; }

/** @param class-string $any */
function mixed_args(string $any): void {
    $o = pair(Good::class, $any);
    /** @mir-check $o is Good */
    echo get_class($o);
}

function violating(): void {
    register(Bad::class);
//  ^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'Bad' does not satisfy bound 'Named'
}

/** @param class-string $any */
function union_arg(string $any, bool $c): void {
    register($c ? Good::class : $any);
}
===expect===
