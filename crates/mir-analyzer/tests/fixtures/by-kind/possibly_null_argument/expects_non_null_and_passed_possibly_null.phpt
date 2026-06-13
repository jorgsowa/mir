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
MissingParamType@5:14-5:20: Parameter $mixed of foo() has no type annotation
UnusedParam@5:14-5:20: Parameter $mixed is never used
