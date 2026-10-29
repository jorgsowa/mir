===description===
A try body that ends in `return` or `throw` may throw after its assignments
ran, so `catch` and `finally` see them as possibly defined, a by-reference
argument included, not as never defined.
===file===
<?php
final class Kernel
{
    public function handle(): int
    {
        try {
            return $this->raw($meta);
        } catch (\Throwable $e) {
            return $this->finish($meta);
//                               ^^^^^ PossiblyUndefinedVariable: Variable $meta might not be defined
        }
    }

    private function raw(?int &$meta = null): int
    {
        $meta = 1;
        return 1;
    }

    private function finish(?int $meta): int
    {
        return (int) $meta;
    }
}

function thrown(): string
{
    try {
        $step = 'parse';
        throw new \RuntimeException('x');
    } catch (\RuntimeException $e) {
        return $step;
//             ^^^^^ PossiblyUndefinedVariable: Variable $step might not be defined
    }
}

function in_finally(): int
{
    try {
        $n = strlen('ab');
        return $n;
    } finally {
        echo $n;
//           ^^ PossiblyUndefinedVariable: Variable $n might not be defined
    }
}

function never_assigned(): int
{
    try {
        return strlen('ab');
    } catch (\Exception $e) {
        return $missing;
//             ^^^^^^^^ UndefinedVariable: Variable $missing is not defined
//      ^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'int'
    }
}
