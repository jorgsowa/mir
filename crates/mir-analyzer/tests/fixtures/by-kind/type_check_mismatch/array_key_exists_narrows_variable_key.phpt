===description===
`array_key_exists($key, $map)` narrows `$key` to the keys `$map` can hold:
a string-keyed map drops the int half of `int|string`, a list keeps only
ints, and a sealed shape turns a `string` key into its literal keys.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesString(string $value): void {}

/** @param array<string, int> $values */
function stringKeys(int|string $key, array $values): void {
    if (array_key_exists($key, $values)) {
        /** @mir-check $key is string */
        takesString($key);
    }
    takesString($key);
//              ^^^^ PossiblyInvalidArgument: Argument $value of takesString() expects 'string', possibly different type 'string|int' provided
}

/** @param list<string> $values */
function listKeys(int|string $key, array $values): void {
    if (array_key_exists($key, $values)) {
        /** @mir-check $key is int */
        $_ = $key;
//      ^^^^^^^^^^ TypeCheckMismatch: Type of $key is expected to be int, got int|string
    }
}

function shapeKeys(string $key): void {
    $values = ['a' => 1, 'b' => 2];
    if (array_key_exists($key, $values)) {
        /** @mir-check $key is "a"|"b" */
        $_ = $key;
    }
}

/** @param array<string, int> $values */
function numericStringKept(string $key, array $values): void {
    if (key_exists($key, $values)) {
        /** @mir-check $key is string */
        $_ = $key;
    }
}
