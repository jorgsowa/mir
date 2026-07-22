===description===
`self<K, V>` with multiple comma-separated generic args in a self-out
annotation must substitute each method template independently — guards the
`split_generics` call in the self-out parser against only handling the
single-arg case.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template K
 * @template V
 */
final class Pair
{
    /**
     * @param K $k
     * @param V $v
     */
    public function __construct(public mixed $k, public mixed $v)
    {
    }

    /**
     * @template K2
     * @template V2
     * @param K2 $k
     * @param V2 $v
     * @psalm-self-out self<K2, V2>
     */
    public function replaceBoth($k, $v): void
    {
    }
}

function test(): void {
    $pair = new Pair(1, true);
    $pair->replaceBoth("a", 3.14);
    /** @mir-check $pair is Pair<string, float> */
    $_ = 1;
}
===expect===
