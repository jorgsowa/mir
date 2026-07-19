===description===
is_a($obj->prop, X::class, true)'s allow_string branch must not collapse
an unrelated-but-still-valid property type to empty; mark_diverges=false
means "leave untouched", not "narrow to bottom".
===config===
suppress=MissingConstructor
===file===
<?php
class Foo {}
class Bar {}
class Container {
    public Foo $item;
}
function f(Container $c): void {
    if (is_a($c->item, 'Bar', true)) {
        // Foo and Bar are unrelated concrete classes: no object or string
        // match is possible, but the property must stay Foo, not collapse
        // to an empty/bottom type.
        /** @mir-check $c->item is Foo */
        $_ = $c->item;
    }
}
===expect===
