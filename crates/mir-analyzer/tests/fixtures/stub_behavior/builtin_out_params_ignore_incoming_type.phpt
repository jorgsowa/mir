===description===
A by-ref param with a default in a bundled stub is an out-param: PHP's arginfo
leaves it untyped and never reads the incoming value, so a variable that still
holds something else can be passed (`preg_match($p, $s, $m)` with a string
`$m`). The declared type is what the call writes back. The intl `$offset`
params are read before being updated, so they keep their incoming `int` check.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function reuse(string $subject): int {
    $matches = 'previous';
    preg_match('/(a)/', $subject, $matches);
    /** @mir-check $matches is list<string> */
    $all = 5;
    preg_match_all('/a/', $subject, $all);
    $count = 'none';
    str_replace('a', 'b', $subject, $count);
    /** @mir-check $count is int */
    $percent = 'n/a';
    similar_text('a', 'b', $percent);
    /** @mir-check $percent is float */
    return $count;
}

function offset(NumberFormatter $fmt): void {
    $pos = 'start';
    $fmt->parse('12', NumberFormatter::TYPE_DOUBLE, $pos);
//                                                  ^^^^ InvalidArgument: Argument $offset of parse() expects 'int|null', got '"start"'
}
