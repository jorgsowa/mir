===description===
Array/list pseudo-types, the `T[]`/`T[][]` shorthand, and `iterable`'s
`array|Traversable` decomposition (Display re-collapses the decomposition
back to `iterable`/`iterable<K, V>`, so the cross-form checks below still
match after round-tripping through the parser twice). Also pins down a real
parser bug in the bare `non-empty-list` keyword (see the comment above
`check_non_empty_list_bare_keyword_loses_non_empty_bug`).
===config===
suppress=MissingReturnType,MissingParamType,ForbiddenCode
===file===
<?php
function check_array($x) {
    /**
     * @var array $x
     * @mir-check $x is array<int|string, mixed>
     */
    var_dump($x);
}

function check_non_empty_array($x) {
    /**
     * @var non-empty-array $x
     * @mir-check $x is non-empty-array<int|string, mixed>
     */
    var_dump($x);
}

function check_list($x) {
    /**
     * @var list $x
     * @mir-check $x is list<mixed>
     */
    var_dump($x);
}

// NOTE: this pins down a real parser bug rather than the documented
// semantics — the bare `non-empty-list` keyword arm in
// parser/docblock/types.rs builds a plain `Atomic::TList` instead of
// `Atomic::TNonEmptyList`, so it silently loses the "non-empty" guarantee.
// The generic form `non-empty-list<T>` (parse_generic) is unaffected and
// correctly builds `TNonEmptyList`. Flagged separately; this fixture
// documents current (buggy) behavior rather than the intended one.
function check_non_empty_list_bare_keyword_loses_non_empty_bug($x) {
    /**
     * @var non-empty-list $x
     * @mir-check $x is list<mixed>
     */
    var_dump($x);
}

function check_non_empty_list_generic_form($x) {
    /**
     * @var non-empty-list<int> $x
     * @mir-check $x is non-empty-list<int>
     */
    var_dump($x);
}

function check_array_shorthand($x) {
    /**
     * @var int[] $x
     * @mir-check $x is array<int|string, int>
     */
    var_dump($x);
}

function check_array_shorthand_nested($x) {
    /**
     * @var int[][] $x
     * @mir-check $x is array<int|string, array<int|string, int>>
     */
    var_dump($x);
}

function check_iterable($x) {
    /**
     * @var iterable $x
     * @mir-check $x is array<int|string, mixed>|Traversable
     */
    var_dump($x);
}

function check_iterable_generic($x) {
    /**
     * @var iterable<int, string> $x
     * @mir-check $x is array<int, string>|Traversable<int, string>
     */
    var_dump($x);
}
===expect===
