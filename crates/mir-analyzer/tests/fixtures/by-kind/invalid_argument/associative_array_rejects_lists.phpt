===description===
`associative-array` rejects a list, from a literal or a `list<T>` value. `non-empty-associative-array` also rejects `[]`. A native `array` hint beside either docblock type is not `MismatchingDocblockParamType`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
declare(strict_types=1);

/** @param associative-array<int, string> $map */
function assoc(array $map): void {}

/** @param non-empty-associative-array<string, int> $map */
function nonEmptyAssoc(array $map): void {}

assoc([5 => 'a', 9 => 'b']);
assoc(['a', 'b']);
//    ^^^^^^^^^^ InvalidArgument: Argument $map of assoc() expects 'array<int, string>&array{}', got 'array{0: "a", 1: "b"}'

/** @param list<string> $list */
function fromList(array $list): void {
    assoc($list);
    //    ^^^^^ InvalidArgument: Argument $map of assoc() expects 'array<int, string>&array{}', got 'list<string>'
}

nonEmptyAssoc(['a' => 1]);
nonEmptyAssoc([]);
//            ^^ InvalidArgument: Argument $map of nonEmptyAssoc() expects 'non-empty-array<string, int>&array{}', got 'array{}'
nonEmptyAssoc([1, 2]);
//            ^^^^^^ InvalidArgument: Argument $map of nonEmptyAssoc() expects 'non-empty-array<string, int>&array{}', got 'array{0: 1, 1: 2}'
