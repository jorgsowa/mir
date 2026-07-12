===description===
A final exception class named only in a function's `@throws` docblock tag
must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class MyException extends \RuntimeException {}

/**
 * @throws MyException
 */
function risky(): void {
}

risky();
===expect===
