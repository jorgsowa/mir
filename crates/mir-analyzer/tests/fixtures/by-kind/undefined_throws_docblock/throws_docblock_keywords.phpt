===description===
Docblock keywords in `@throws` annotations are not undefined classes.
===file===
<?php
/**
 * @throws boolean
 * @throws list
 * @throws iterable
 */
function risky(): string {
    return 'x';
}

===expect===
