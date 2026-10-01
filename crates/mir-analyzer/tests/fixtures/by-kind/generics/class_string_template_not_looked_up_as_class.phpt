===description===
class-string<T> over an in-scope template is not resolved as a literal class
===config===
suppress=MixedReturnStatement
===file===
<?php
/**
 * @template E of \BackedEnum
 * @param class-string<E> $enumClass
 * @return Closure(string): ?E
 */
function enumParser(string $enumClass): Closure {
    return function (string $v) use ($enumClass): ?\BackedEnum {
        return $enumClass::tryFrom($v);
    };
}

/**
 * @template T
 * @param class-string<T> $c
 */
function viaArrow(string $c): Closure {
    return fn() => $c::create();
}

/**
 * @template T
 * @param class-string<T> $c
 */
function constAndStatic(string $c): void {
    echo $c::X;
    echo $c::$prop;
    $c::run();
    new $c();
    /** @mir-check $c is class-string<T> */
    $c;
}

/**
 * @template T
 * @param class-string<T> $c
 */
function instanceOfTemplate(object $o, string $c): bool {
    return $o instanceof $c;
}
===expect===
