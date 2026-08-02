===description===
A `@param` referencing the enclosing class's own bare name as a generic type
(`Option<S>`) must resolve the outer name against the current namespace
(self-reference) while still converting the inner `S` to a template param,
not a literal undefined class `S`.
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @template S
 */
abstract class Option {
    /**
     * @param Option<S>|callable|S $value
     */
    public function ensure($value): void {
    }
}
===expect===
