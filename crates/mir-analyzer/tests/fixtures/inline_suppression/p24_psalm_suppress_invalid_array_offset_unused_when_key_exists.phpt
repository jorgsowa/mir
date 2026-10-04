===description===
Negative control: when the key exists there is nothing to silence, so the aliased
suppression is still reported as unused.
===file===
<?php
/** @param array{a: int} $v */
function test(array $v): void {
    /** @psalm-suppress InvalidArrayOffset */
    echo $v['a'];
}
===expect===
UnusedSuppress@5:0-5:0: Suppress annotation for 'InvalidArrayOffset' is never used
