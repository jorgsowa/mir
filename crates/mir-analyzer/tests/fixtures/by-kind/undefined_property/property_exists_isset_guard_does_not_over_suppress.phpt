===description===
Negative counterpart of the property_exists/isset guard fix: the guard
must be scoped to the exact (receiver, property-name) pair proven and to
the branch where it was proven — a different property name, a different
receiver, a mismatched case, or code outside the guarded branch must
still flag UndefinedProperty.
===file===
<?php
class Plain {
    public function guardedNameOnly(): mixed {
        if (property_exists($this, 'extra')) {
            return $this->extra;
        }
        return $this->other;
//                    ^^^^^ UndefinedProperty: Property Plain::$other does not exist
    }

    public function caseSensitive(): mixed {
        if (property_exists($this, 'Extra')) {
            return $this->extra;
//                        ^^^^^ UndefinedProperty: Property Plain::$extra does not exist
        }
        return null;
    }

    public function doesNotLeakPastIf(): mixed {
        if (property_exists($this, 'extra')) {
            return $this->extra;
        }
        return $this->extra;
//                    ^^^^^ UndefinedProperty: Property Plain::$extra does not exist
    }
}
