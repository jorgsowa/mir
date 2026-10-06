===description===
A @param array shape wrapped across multiple lines is still parsed and checked,
not silently dropped to an unchecked parameter.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array{
 *     id: int,
 *     name: string,
 * } $data
 */
function f(array $data): void {}

f("not an array");
//^^^^^^^^^^^^^^ InvalidArgument: Argument $data of f() expects 'array{'id': int, 'name': string}', got '"not an array"'
