===description===
`int<1,255>` is a subtype of `int<0,255>`.
===config===
php_version=8.4
===file===
<?php
/** @param int<0,255> $c */
function channel(int $c): void { echo $c; }
/** @return int<1,255> */
function bright(): int { return 200; }
function run(): void {
    channel(bright());
}
===expect===
