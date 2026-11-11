===description===
The by-reference requirement is checked for named functions and methods passed as first-class callables or by name.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function trimInPlace(string &$value): void {
    $value = trim($value);
}

function trimCopy(string $value): string {
    return trim($value);
}

final class Cleaner {
    public function strip(string &$value): void {
        $value = strip_tags($value);
    }

    public function peek(string $value): void {}
}

/** @param callable(string &$value): mixed $normalizer */
function normalize(callable $normalizer): void {}

function run(Cleaner $cleaner): void {
    normalize(trimInPlace(...));
    normalize('trimInPlace');
    normalize($cleaner->strip(...));
    normalize(trimCopy(...));
//            ^^^^^^^^^^^^^ InvalidArgument: Argument $normalizer of normalize() expects 'callable taking parameter #1 by reference', got 'callable taking parameter #1 by value'
    normalize('trimCopy');
//            ^^^^^^^^^^ InvalidArgument: Argument $normalizer of normalize() expects 'callable taking parameter #1 by reference', got 'callable taking parameter #1 by value'
    normalize($cleaner->peek(...));
//            ^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $normalizer of normalize() expects 'callable taking parameter #1 by reference', got 'callable taking parameter #1 by value'
}
