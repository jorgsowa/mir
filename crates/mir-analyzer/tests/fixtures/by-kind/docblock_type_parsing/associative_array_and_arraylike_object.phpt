===description===
`associative-array<K, V>`/`non-empty-associative-array` and
`arraylike-object<K, V>` are encoded as intersections (`TArray & TKeyedArray`
marker, or `ArrayAccess & Countable & Traversable`). `Display` has no
special case that reprints these intersections back as the pseudo-type
keyword, so they render as their literal decomposition rather than as
`associative-array`/`arraylike-object` — this looks like a real Display gap
rather than intended behavior; flagged separately rather than guessed at
here, since getting the exact rendered string wrong would make this fixture
assert the bug instead of documenting it. Only the generic (parameterized)
form is checked, since it doesn't hit the display-side `TObject` sibling
quirk that the bare keyword's expansion adds.
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
function check_associative_array_generic($x) {
    /**
     * @var associative-array<string, int> $x
     * @mir-check $x is array<string, int>&array{}
     */
    var_dump($x);
}

function check_arraylike_object_generic($x) {
    /**
     * @var arraylike-object<int, string> $x
     * @mir-check $x is \ArrayAccess<int, string>&\Countable&\Traversable<int, string>
     */
    var_dump($x);
}
