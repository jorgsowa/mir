===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    /**
     * @suppress PossiblyInvalidArgument
     */
    takesInt(getResult());
}
===expect===
