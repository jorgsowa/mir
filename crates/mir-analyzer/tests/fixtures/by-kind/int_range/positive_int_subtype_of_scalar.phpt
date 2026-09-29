===description===
`positive-int` is a subtype of `scalar`.
===config===
php_version=8.4
===file===
<?php
/** @param scalar $value */
function equalTo($value): void { echo $value; }
/** @return positive-int */
function count_(): int { return 1; }
function run(): void {
    equalTo(count_());
}
===expect===
