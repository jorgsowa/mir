===description===
Regression (laravel/framework): `Numeric` is a valid class name in PHP (only
bool/int/float/string/true/false/null/void/iterable/object/mixed/never are
reserved). The underlying php-rs-parser over-broadly rejected `numeric` (and
`resource`); mir now drops that spurious reserved-class ParseError so the
declaration analyzes normally.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedClass
===file===
<?php
class Numeric {
    public function __toString(): string {
        return 'numeric';
    }
}
===expect===
