===description===
Negative control for the template-aware closure/arrow-fn own-docblock fix:
a name in a closure's own `@return` docblock that matches no enclosing
class/method `@template` must still resolve as an ordinary (and namespace-
qualified) class reference, same as before the fix — so the resulting
declared/actual mismatch is still flagged, exactly as it would be for any
other non-existent docblock class.
===config===
suppress=UnusedParam
===file===
<?php
namespace PhpOption;

/** @template T */
abstract class Option {
    /** @return T */
    abstract public function get();

    public static function lift(array $args) {
        return array_map(
            /** @return NotARealClass */
            static function (self $o) {
                return $o->get();
            },
            $args
        );
    }
}
===expect===
MixedReturnStatement@13:16-13:33: Cannot return a mixed type from function with declared return type 'PhpOption\NotARealClass'
