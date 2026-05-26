===description===
suppressed via psalm suppress
===file===
<?php
function test(): void {
    /**
     * @psalm-suppress UndefinedFunction
     */
    noSuchFunction();
}
===expect===
