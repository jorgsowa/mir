===description===
A `literal-string` value in an array shape may or may not satisfy a refined
string key, like a plain `string`, so it is a coercion, not an invalid argument.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{file: non-falsy-string|null, name?: string} $metadata */
function register(array $metadata): void {}

function run(): void {
    register(['file' => __FILE__, 'name' => 'block']);
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ArgumentTypeCoercion: Argument $metadata of register() expects 'array{'file': non-falsy-string|null, 'name'?: string}', got 'array{'file': literal-string, 'name': "block"}' — coercion may fail at runtime
}
