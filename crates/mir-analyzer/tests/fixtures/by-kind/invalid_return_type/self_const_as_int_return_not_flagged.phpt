===description===
self::INT_CONST returned from a function declared : int is not flagged.
The constant's literal type is resolved instead of falling back to mixed.
===file===
<?php
class Status {
    const ACTIVE   = 1;
    const INACTIVE = 0;

    public function getActive(): int {
        return self::ACTIVE;
    }

    public function getInactive(): int {
        return self::INACTIVE;
    }

    public static function defaultStatus(): int {
        return static::ACTIVE;
    }
}
===expect===
