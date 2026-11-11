===description===
`@phan-side-effect-free` is read as `@pure`, on a function, a method or a whole class. `@phpstan-all-methods-pure` makes every method of the class pure unless it is marked `@phpstan-impure`.
===file===
<?php
/** @phan-side-effect-free */
function claimsSideEffectFree(int $value): int
{
    echo 'side effect';
//  ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function echo() in a @pure function
    return $value + 1;
}

/** @phpstan-all-methods-pure */
final class PureBox
{
    public function add(int $left, int $right): int
    {
        echo 'side effect';
//      ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function echo() in a @pure function
        return $left + $right;
    }

    /** @phpstan-impure */
    public function log(int $value): int
    {
        echo 'logged';
        return $value;
    }
}

/** @phan-side-effect-free */
final class SideEffectFreeBox
{
    public function add(int $left, int $right): int
    {
        echo 'side effect';
//      ^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function echo() in a @pure function
        return $left + $right;
    }
}
