===description===
No FP InvalidReturnType when declared return has template params nested inside array/list type params of a generic class
===config===
suppress=UnusedParam,UnusedVariable,UnusedFunction
===file:Result.php===
<?php

/**
 * @template L
 * @template R
 */
class Result
{
    /**
     * @param list<L> $lefts
     * @param list<R> $rights
     */
    public function __construct(
        public array $lefts,
        public array $rights,
    ) {
        echo count($lefts) + count($rights);
    }
}
===file:partition.php===
<?php

/**
 * @template L
 * @template R
 * @return Result<list<L>, list<R>>
 */
function makeResultConcrete(): mixed
{
    // Actual type is Result<list<string>, list<int>> — incompatible with declared
    // Result<list<L>, list<R>> if L/R aren't detected as templates.
    // With the fix, declared_return_has_template correctly finds TTemplateParam inside
    // list<L> and suppresses the error.
    /** @var Result<list<string>, list<int>> $r */
    $r = new Result([], []);
    return $r;
}

/**
 * @template L
 * @template R
 * @return Result<non-empty-array<int, L>, array<int, R>>
 */
function makeResultNestedArray(): mixed
{
    // Same pattern with non-empty-array<int, L> — tests array (not just list) nesting.
    /** @var Result<non-empty-array<int, string>, array<int, int>> $r */
    $r = new Result([], []);
    return $r;
}
===expect===
