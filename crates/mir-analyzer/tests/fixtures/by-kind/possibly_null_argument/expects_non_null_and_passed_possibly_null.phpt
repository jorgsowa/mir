===description===
Expects non null and passed possibly null
===file===
<?php
/**
 * @param mixed|null $mixed_or_null
 */
function foo($mixed, $mixed_or_null): void {
    /**
     * @suppress MixedArgument
     */
    new Exception($mixed_or_null);
}
===expect===
UnusedPsalmSuppress@9:0-9:0: Suppress annotation for 'MixedArgument' is never used
