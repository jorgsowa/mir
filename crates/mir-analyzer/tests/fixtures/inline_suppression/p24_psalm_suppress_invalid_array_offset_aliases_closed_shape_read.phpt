===description===
Psalm's `InvalidArrayOffset` covers a read of an undeclared key on a closed shape,
which mir reports as `NonExistentArrayOffset`; the suppression silences it and is
not flagged unused. The suppressed read still evaluates to mixed.
===file===
<?php
class Holder {
    /** @var array{a: int} */
    public array $config = ['a' => 1];
}
function test(Holder $h): void {
    /** @psalm-suppress InvalidArrayOffset */
    $x = $h->config['missing'];
    /** @mir-check $x is mixed */
    echo $x;
}
===expect===
MixedAssignment@8:4-8:30: Variable $x is assigned a mixed type
