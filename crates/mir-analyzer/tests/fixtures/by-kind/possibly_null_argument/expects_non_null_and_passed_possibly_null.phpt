===description===
Expects non null and passed possibly null
===file===
<?php
/**
 * @param mixed|null $mixed_or_null
 */
function foo($mixed, $mixed_or_null): void {
//           ^^^^^^ MissingParamType: Parameter $mixed of foo() has no type annotation
//           ^^^^^^ UnusedParam: Parameter $mixed is never used
    /**
     * @suppress MixedArgument
     */
    new Exception($mixed_or_null);
}
