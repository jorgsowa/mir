===description===
str_replace accepts scalar, Stringable and null replacement elements (PHP casts them), but not nested arrays
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Label implements Stringable
{
    public function __toString(): string
    {
        return 'label';
    }
}

function replace(Label $label): void {
    $a = str_replace('a', [0, 'b', 'c'], 'subject');
    $b = str_replace(['a', 'b'], [1.5, null, true], 'subject');
    $c = str_replace(['a'], [$label], ['x', 'y']);
    $d = str_replace('a', 'b', 'subject');
    /** @mir-check $a is string */
    /** @mir-check $c is array<int, string> */
}

function rejectsNested(): void {
    str_replace('a', [['nested']], 'subject');
//                   ^^^^^^^^^^^^ InvalidArgument: Argument $replace of str_replace() expects 'string|array<int|string, scalar|Stringable|null>', got 'array{0: array{0: "nested"}}'
}
