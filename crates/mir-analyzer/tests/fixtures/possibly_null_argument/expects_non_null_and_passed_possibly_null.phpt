===description===
expectsNonNullAndPassedPossiblyNull
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
PossiblyNullArgument
===ignore===
TODO
