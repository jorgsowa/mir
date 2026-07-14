===description===
FP: a call that passes named arguments in a different textual order than the
callee declares them must still bind each argument's template param to its
OWN declared parameter, not to whichever parameter sits at that argument's
syntactic position. Covers a free function, a method, and a constructor.
===config===
suppress=UnusedVariable,MissingPropertyType
===file===
<?php
/**
 * @template K of int
 * @template V of string
 * @param K $key
 * @param V $value
 * @return array{0: K, 1: V}
 */
function makePair($key, $value): array {
    return [$key, $value];
}

// Valid: key=42 (int, satisfies K of int), value="hello" (string, satisfies
// V of string) — just passed in reversed textual order via named arguments.
makePair(value: "hello", key: 42);

class Factory {
    /**
     * @template K of int
     * @template V of string
     * @param K $key
     * @param V $value
     * @return array{0: K, 1: V}
     */
    public function makePair($key, $value): array {
        return [$key, $value];
    }
}
(new Factory())->makePair(value: "hello", key: 42);

/**
 * @template K of int
 * @template V of string
 */
class Pair {
    /**
     * @param K $key
     * @param V $value
     */
    public function __construct(public $key, public $value) {}
}
new Pair(value: "hello", key: 42);

// A real bound violation must still be caught regardless of argument order:
// value=42 is an int, which doesn't satisfy V's `of string` bound.
makePair(value: 42, key: 1);
===expect===
InvalidTemplateParam@46:0-46:27: Template type 'V' inferred as '42' does not satisfy bound 'string'
