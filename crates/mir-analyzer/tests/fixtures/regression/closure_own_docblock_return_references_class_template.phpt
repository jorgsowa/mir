===description===
A closure literal's OWN leading `@return T` docblock, where `T` matches the
enclosing class's `@template`, must resolve to a `TTemplateParam` just like
the closure body already does — not get treated as an ordinary (and
namespace-mis-qualified) class reference named "T".
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
            /** @return T */
            static function (self $o) {
                return $o->get();
            },
            $args
        );
    }
}
===expect===
