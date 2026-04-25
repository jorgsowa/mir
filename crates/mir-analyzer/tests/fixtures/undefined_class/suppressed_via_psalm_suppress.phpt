===file===
<?php
function test(): void {
    /**
     * @psalm-suppress UndefinedClass
     */
    new NoSuchClass();
}
===expect===
