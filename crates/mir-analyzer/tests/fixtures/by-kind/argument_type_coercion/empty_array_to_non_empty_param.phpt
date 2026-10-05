===description===
A literal [] passed to a non-empty list/array param is an ArgumentTypeCoercion; a non-empty
literal or an empty array for a possibly-empty param is accepted.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-list<int> $l */
function takes_non_empty_list(array $l): void {}
/** @param non-empty-array<string, int> $a */
function takes_non_empty_array(array $a): void {}
/** @param list<int>|null $l */
function takes_maybe_empty(?array $l): void {}

function run(): void {
    takes_non_empty_list([]);
//                       ^^ ArgumentTypeCoercion: Argument $l of takes_non_empty_list() expects 'non-empty-list<int>', got 'array{}' — coercion may fail at runtime
    takes_non_empty_array([]);
//                        ^^ ArgumentTypeCoercion: Argument $a of takes_non_empty_array() expects 'non-empty-array<string, int>', got 'array{}' — coercion may fail at runtime
    takes_non_empty_list([1]);
    takes_maybe_empty([]);
}
===expect===
