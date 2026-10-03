===description===
`positive-int` is a subtype of `scalar`.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
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
