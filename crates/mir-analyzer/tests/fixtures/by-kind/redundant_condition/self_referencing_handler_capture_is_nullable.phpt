===description===
A by-ref capture of the variable the enclosing assignment defines may still be null inside the closure (`$prev = set_error_handler(function () use (&$prev) {…})` returns null without a previous handler), so testing it is not redundant. Calling a recursive closure through it stays quiet.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function install(): void {
    $previous = set_error_handler(static function (int $level, string $message) use (&$previous): bool {
        return $previous ? (bool) $previous($level, $message) : false;
    });
}

function guarded(): void {
    $previous = set_exception_handler(static function (\Throwable $e) use (&$previous): void {
        if ($previous) {
            $previous($e);
        }
    });
}

/** @param array<int, mixed> $tree */
function walk(array $tree): int {
    $count = 0;
    $visit = function (array $node) use (&$visit, &$count): void {
        foreach ($node as $child) {
            $count++;
            if (is_array($child)) {
                $visit($child);
            }
        }
    };
    $visit($tree);
    return $count;
}

/**
 * @param list<mixed> $values
 * @return list<mixed>
 */
function passedAsCallback(array $values): array {
    $normalize = function (mixed $value) use (&$normalize): mixed {
        return is_array($value) ? array_map($normalize, $value) : $value;
    };
    return array_map($normalize, $values);
}
