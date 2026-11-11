===description===
A plain string may not be one of the literals a parameter accepts.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param 'asc'|'desc' $direction */
function sortBy(string $direction): void {}
/** @param 'asc'|'desc'|null $direction */
function maybeSortBy(?string $direction): void {}

/** @param non-empty-string $name */
function run(string $input, string $name, ?string $optional): void {
    sortBy($input);
//         ^^^^^^ ArgumentTypeCoercion: Argument $direction of sortBy() expects '"asc"|"desc"', got 'string' — coercion may fail at runtime
    sortBy($name);
//         ^^^^^ ArgumentTypeCoercion: Argument $direction of sortBy() expects '"asc"|"desc"', got 'non-empty-string' — coercion may fail at runtime
    maybeSortBy($optional);
//              ^^^^^^^^^ ArgumentTypeCoercion: Argument $direction of maybeSortBy() expects '"asc"|"desc"|null', got 'string|null' — coercion may fail at runtime
}
