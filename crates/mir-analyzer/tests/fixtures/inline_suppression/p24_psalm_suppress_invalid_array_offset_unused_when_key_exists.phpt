===description===
Negative control: when the key exists there is nothing to silence, so the aliased
suppression is still reported as unused.
===file===
<?php
/** @param array{a: int} $v */
function test(array $v): void {
    /** @psalm-suppress InvalidArrayOffset */
//                      ^^^^^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'InvalidArrayOffset' is never used
    echo $v['a'];
}
===expect===
