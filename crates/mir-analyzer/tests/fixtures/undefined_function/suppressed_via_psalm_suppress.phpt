===source===
<?php
function test(): void {
    /**
     * @psalm-suppress UndefinedFunction
     */
    noSuchFunction();
}
===expect===
