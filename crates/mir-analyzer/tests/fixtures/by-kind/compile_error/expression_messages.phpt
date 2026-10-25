===description===
Expression errors the parser reports use PHP's text.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
function f(int|string|INT $x): int {
//                    ^^^ CompileError: Duplicate type int is redundant [duplicate-type]
    $list = [$x];
    $y = $list[];
//  ^^ UnusedVariable: Variable $y is never read
//       ^^^^^^^ CompileError: Cannot use [] for reading [append-outside-write]
    $z = (unset) $x;
//  ^^ UnusedVariable: Variable $z is never read
//        ^^^^^ CompileError: The (unset) cast is no longer supported [unset-cast]
    return match ($x) {
        default => 1,
        default => 2,
//      ^^^^^^^ CompileError: Match expressions may only contain one default arm [duplicate-match-default]
    };
}
