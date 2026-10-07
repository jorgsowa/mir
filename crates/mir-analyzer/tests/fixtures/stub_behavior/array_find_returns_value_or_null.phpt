===description===
`array_find`/`array_find_key` return the array's value/key type or null; with an untyped array the
callback's parameter type still bounds the found value.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, array{type: string, value: string}> $fields */
function keyOf(array $fields): ?string {
    $field = array_find($fields, static fn (array $f) => $f['type'] === 'country');
    /** @mir-check $field is array{type: string, value: string}|null */
    $key = array_find_key($fields, static fn (array $f) => $f['value'] !== '');
    /** @mir-check $key is string|null */
    return $field === null ? null : $key;
}

function valueOf(mixed $fields): mixed {
    $field = array_find($fields, static fn (array $f) => $f['type'] === 'country');
    /** @mir-check $field is array<array-key, mixed>|null */
    return $field === null ? null : $field['value'];
}
