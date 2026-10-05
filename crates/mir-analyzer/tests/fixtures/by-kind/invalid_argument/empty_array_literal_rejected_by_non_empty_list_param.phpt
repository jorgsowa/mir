===description===
An empty array literal `[]` (a closed, zero-property shape) is not a guaranteed
`non-empty-list<T>`/`non-empty-array<K,V>` argument (ArgumentTypeCoercion) — guards against
`array_list_compatible`'s `.all()` over an empty shape's properties being
vacuously true and silently accepting it.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-list<int> $values */
function takesNonEmptyList(array $values): void {}

/** @param non-empty-array<string, int> $counts */
function takesNonEmptyArray(array $counts): void {}

takesNonEmptyList([1, 2, 3]);
takesNonEmptyList([]);
//                ^^ ArgumentTypeCoercion: Argument $values of takesNonEmptyList() expects 'non-empty-list<int>', got 'array{}' — coercion may fail at runtime

takesNonEmptyArray(['a' => 1]);
takesNonEmptyArray([]);
//                 ^^ ArgumentTypeCoercion: Argument $counts of takesNonEmptyArray() expects 'non-empty-array<string, int>', got 'array{}' — coercion may fail at runtime
===expect===
