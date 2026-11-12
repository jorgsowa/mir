===description===
Without strict_types, a scalar of another type passed to a general int, float, string or bool parameter is InvalidScalarArgument, since PHP converts it when it can. A string that can never be numeric, a literal-typed parameter, a non-scalar value and a strict file stay InvalidArgument.
===file:loose.php===
<?php
/** @param 'a'|'b' $mode */
function mode(string $mode): string { return $mode; }
function page(int $n): int { return $n; }
function flag(bool $on): bool { return $on; }

function juggles(string $input, int $count, float $ratio): void {
    page($input);
//       ^^^^^^ InvalidScalarArgument: Argument $n of page() expects 'int', got scalar 'string'
    flag($count);
//       ^^^^^^ InvalidScalarArgument: Argument $on of flag() expects 'bool', got scalar 'int'
    page(false);
//       ^^^^^ InvalidScalarArgument: Argument $n of page() expects 'int', got scalar 'false'
    page('12');
//       ^^^^ InvalidScalarArgument: Argument $n of page() expects 'int', got scalar '"12"'
    page('twelve');
//       ^^^^^^^^ InvalidArgument: Argument $n of page() expects 'int', got '"twelve"'
    mode(1);
//       ^ InvalidArgument: Argument $mode of mode() expects '"a"|"b"', got '1'
    page([]);
//       ^^ InvalidArgument: Argument $n of page() expects 'int', got 'array{}'
    flag($ratio);
//       ^^^^^^ InvalidScalarArgument: Argument $on of flag() expects 'bool', got scalar 'float'
}
===file:strict.php===
<?php
declare(strict_types=1);

function strictPage(string $input): int {
    return page($input);
//              ^^^^^^ InvalidArgument: Argument $n of page() expects 'int', got 'string'
}
