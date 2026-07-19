===description===
`atom_excluded_from_is_iterable_or_countable` and the false-branch filters
in `narrow_var_to_specific_class`/`narrow_prop_to_specific_class` only
matched `Atomic::TNamedObject`, inconsistent with `narrow_strict_subclass_of`
in the same file, which already treats `TNamedObject`/`TSelf`/
`TStaticObject`/`TParent` as equivalent receiver atoms. `is_countable()`'s
false branch and `get_class() !== 'X'`'s exact-class exclusion both
silently no-op'd when the receiver was a `self`/`static`-typed atom.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class OtherThing {}

class Box implements Countable {
    public function count(): int {
        return 0;
    }

    /** @param self|OtherThing $x */
    public function excludesSelfWhenNotCountable($x): void {
        if (!is_countable($x)) {
            /** @mir-check $x is OtherThing */
            $_ = 1;
        }
    }
}

final class FinalBox {
    /** @param self|OtherThing $x */
    public function excludesSelfWhenGetClassMismatches($x): void {
        if (get_class($x) !== 'FinalBox') {
            /** @mir-check $x is OtherThing */
            $_ = 1;
        }
    }
}
===expect===
