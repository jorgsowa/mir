===description===
An empty array literal `[]` (a closed, zero-property shape) is not a valid
`non-empty-list<T>`/`non-empty-array<K,V>` argument — guards against
`array_list_compatible`'s `.all()` over an empty shape's properties being
vacuously true and silently accepting it.
===config===
suppress=MissingParamType,UnusedParam
===file===
<?php
/** @param non-empty-list<int> $values */
function takesNonEmptyList(array $values): void {}

/** @param non-empty-array<string, int> $counts */
function takesNonEmptyArray(array $counts): void {}

takesNonEmptyList([1, 2, 3]);
takesNonEmptyList([]);

takesNonEmptyArray(['a' => 1]);
takesNonEmptyArray([]);
===expect===
InvalidArgument@9:18-9:20: Argument $values of takesNonEmptyList() expects 'non-empty-list<int>', got 'array{}'
InvalidArgument@12:19-12:21: Argument $counts of takesNonEmptyArray() expects 'non-empty-array<string, int>', got 'array{}'
