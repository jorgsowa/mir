===description===
Both members of a `@throws A|B` union tag must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class FirstException extends \RuntimeException {}
final class SecondException extends \RuntimeException {}

/**
 * @throws FirstException|SecondException
 */
function risky(): void {
}

risky();
===expect===
