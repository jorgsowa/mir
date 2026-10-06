===description===
Callbacks of usort-family functions and array_reduce may declare fewer parameters than are passed; requiring more is invalid
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function fewer_params(): void {
    $items = [3, 1, 2];
    usort($items, static fn(int $a): int => $a > 1 ? 1 : 0);
    uasort($items, static fn(int $a): int => $a);
    uksort($items, static fn(): int => 0);
    usort($items, static fn(int $a, int $b = 0, int ...$rest): int => $a <=> $b);
    array_reduce($items, static fn(int $carry): int => $carry, 0);
    array_reduce($items, static function (): int { return 1; }, 0);
}

function exact_params(): void {
    $items = [3, 1, 2];
    usort($items, static fn(int $a, int $b): int => $a <=> $b);
    array_reduce($items, static fn(int $carry, int $item): int => $carry + $item, 0);
}

function too_many_required_params(): void {
    $items = [3, 1, 2];
    usort($items, static fn(int $a, int $b, int $c): int => 0);
//                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $callback of usort() expects 'callable accepting at most 2 arguments', got 'callable accepting 3 arguments'
    array_reduce($items, static fn(int $a, int $b, int $c): int => 0, 0);
//                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $callback of array_reduce() expects 'callable accepting at most 2 arguments', got 'callable accepting 3 arguments'
}
===expect===
