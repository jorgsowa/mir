===description===
Negative control: the alias only silences the annotated statement; an identical read
on the following statement is still reported.
===file===
<?php
/** @param array{a: int} $v */
function test(array $v): void {
    /** @psalm-suppress InvalidArrayOffset */
    echo $v['missing'];
    echo $v['missing'];
//          ^^^^^^^^^ NonExistentArrayOffset: Array offset 'missing' does not exist
}
