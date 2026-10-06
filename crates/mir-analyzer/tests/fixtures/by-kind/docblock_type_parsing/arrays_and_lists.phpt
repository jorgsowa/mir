===description===
Array/list pseudo-types, the `T[]`/`T[][]` shorthand, and `iterable`'s
`array|Traversable` decomposition (Display re-collapses the decomposition
back to `iterable`/`iterable<K, V>`, so the cross-form checks below still
match after round-tripping through the parser twice).
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
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

function check_non_empty_list_bare_keyword($x) {
    /**
     * @var non-empty-list $x
     * @mir-check $x is non-empty-list<mixed>
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
