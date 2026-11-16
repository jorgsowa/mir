===description===
A runtime `string` passed where a `class-string`, `interface-string`, `enum-string` or `trait-string` is expected is a coercion, like `string` to `non-empty-string`. Class constants, `get_class()` and `get_parent_class()` results, a parameter that also accepts `string`, and `mixed` keep their existing reports; a literal that cannot name a class stays `InvalidArgument`.
===file===
<?php
/** @param class-string $c */
function cs(string $c): void { echo $c; }

/** @param class-string<\Countable> $c */
function csc(string $c): void { echo $c; }

/** @param interface-string $c */
function ifs(string $c): void { echo $c; }

/** @param enum-string $c */
function ens(string $c): void { echo $c; }

/** @param trait-string $c */
function trs(string $c): void { echo $c; }

/** @param non-empty-string $c */
function nes(string $c): void { echo $c; }

/** @param class-string|string $c */
function csOrString(string $c): void { echo $c; }

/** @param non-empty-string $nonEmpty */
function f(string $s, string $nonEmpty, mixed $m): void {
    cs($s);
//     ^^ ArgumentTypeCoercion: Argument $c of cs() expects 'class-string', got 'string' — coercion may fail at runtime
    csc($s);
//      ^^ ArgumentTypeCoercion: Argument $c of csc() expects 'class-string<Countable>', got 'string' — coercion may fail at runtime
    ifs($s);
//      ^^ ArgumentTypeCoercion: Argument $c of ifs() expects 'interface-string', got 'string' — coercion may fail at runtime
    ens($s);
//      ^^ ArgumentTypeCoercion: Argument $c of ens() expects 'enum-string', got 'string' — coercion may fail at runtime
    trs($s);
//      ^^ ArgumentTypeCoercion: Argument $c of trs() expects 'trait-string', got 'string' — coercion may fail at runtime
    nes($s);
//      ^^ ArgumentTypeCoercion: Argument $c of nes() expects 'non-empty-string', got 'string' — coercion may fail at runtime
    cs($nonEmpty);
//     ^^^^^^^^^ ArgumentTypeCoercion: Argument $c of cs() expects 'class-string', got 'non-empty-string' — coercion may fail at runtime
    csOrString($s);
    cs(ArrayObject::class);
    csc(ArrayObject::class);
    cs($m);
//     ^^ MixedArgument: Argument $c of cs() is mixed
    cs('not a class');
//     ^^^^^^^^^^^^^ InvalidArgument: Argument $c of cs() expects 'class-string', got '"not a class"'
}

/**
 * @template T of object
 * @param T $templated
 */
function fromObjects(object $obj, ArrayObject|Exception $either, ?Exception $maybe, object $templated): void {
    cs(get_class($obj));
    cs(get_class($either));
    cs(get_class($templated));
    if ($maybe !== null) {
        cs(get_class($maybe));
    }
    $parent = get_parent_class($obj);
    if ($parent) {
        cs($parent);
    }
}
