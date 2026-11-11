===description===
The literal `'string[]'` cannot spell a class name: it is rejected as a class-string, not looked up as a class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
/** @param class-string $type */
function deserialize(string $type, string $payload): mixed { return null; }
function run(string $payload): mixed {
    return deserialize('string[]', $payload);
//                     ^^^^^^^^^^ InvalidArgument: Argument $type of deserialize() expects 'class-string', got '"string[]"'
}
