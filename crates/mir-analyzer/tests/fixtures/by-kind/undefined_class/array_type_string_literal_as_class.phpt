===description===
FALSE POSITIVE reproducer. Valid PHP: The string literal `'string[]'` is a runtime value, not a class name.
Expected: no issue.
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
    // expect: UndefinedClass "string[]" (array-type string parsed as a class)
    return deserialize('string[]', $payload);
}
===expect===
