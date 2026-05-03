===description===
suppressed via psalm suppress
===file===
<?php
function test(): void {
    /**
     * @psalm-suppress UndefinedClass
     */
    new NoSuchClass();
}
===expect===
===ignore===
TODO
