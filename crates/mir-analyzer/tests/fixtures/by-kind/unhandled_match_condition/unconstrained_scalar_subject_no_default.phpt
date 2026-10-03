===description===
A match() on a plain scalar subject with no default arm can throw UnhandledMatchError for any value the arms don't list; it is reported when an arm is not a literal.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function no_default(int $x, int $y): string {
    return match ($x) { 1 => 'one', $y => 'two' };
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnhandledMatchCondition: Unhandled match condition: possibly-unmatched value of type 'int'
}

function with_default(int $x): string {
    return match ($x) { 1 => 'one', 2 => 'two', default => 'other' };
}

function float_subject(float $x): string {
    return match ($x) { 1.5 => 'a', 2.5 => 'b' };
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnhandledMatchCondition: Unhandled match condition: possibly-unmatched value of type 'float'
}
===expect===
