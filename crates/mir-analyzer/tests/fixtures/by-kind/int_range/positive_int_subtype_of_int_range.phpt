===description===
`positive-int` is a subtype of `int<0,max>`.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
/** @param int<0,max> $n */
function sleepish(int $n): void { echo $n; }
/** @return positive-int */
function size(): int { return 3; }
function run(): void {
    sleepish(size());
}
