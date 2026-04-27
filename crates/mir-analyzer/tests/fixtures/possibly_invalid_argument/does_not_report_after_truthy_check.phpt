===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    $result = getResult();
    if ($result) {
        takesInt($result);
    }
}
===expect===
