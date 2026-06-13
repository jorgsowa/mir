===description===
Laravel FP (laravel/framework): `Numeric` is a valid class name in PHP (only
bool/int/float/string/true/false/null/void/iterable/object/mixed/never are
reserved). mir's reserved-word list wrongly includes `Numeric`, producing a
ParseError on `class Numeric`. Ignored pending fix — see ROADMAP §1.4.
===ignore===
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
