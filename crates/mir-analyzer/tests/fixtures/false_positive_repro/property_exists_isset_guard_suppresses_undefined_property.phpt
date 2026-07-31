===description===
FP-L18a: `property_exists($this, 'x')` / `isset($this->x)` record no
property fact at all — a dynamic/BC property proven present by either
guard still flags UndefinedProperty on a later `$this->x` read inside
the guarded block.
===config===
suppress=MissingConstructor
===file===
<?php
class BackwardsCompatShim {
    public function readViaPropertyExists(): mixed {
        if (property_exists($this, 'extra')) {
            return $this->extra;
        }
        return null;
    }

    public function readViaIsset(): mixed {
        if (isset($this->extra)) {
            return $this->extra;
        }
        return null;
    }

    public function earlyExit(): mixed {
        if (!property_exists($this, 'extra')) {
            return null;
        }
        return $this->extra;
    }
}

function readViaPropertyExistsOnVar(BackwardsCompatShim $obj): mixed {
    if (property_exists($obj, 'extra')) {
        return $obj->extra;
    }
    return null;
}
===expect===
