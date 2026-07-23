===description===
unset(self::$store[$k]) mutates a static property's contents just as
much as unset($this->arr['key']) does, but the unset chain-walk only
ever matched a PropertyAccess base -- StaticPropertyAccess fell through
to the catch-all _ => break, unlike the plain static-property
array-index-WRITE case which is already checked.
===file===
<?php
class Cache {
    private static array $store = [];

    /** @pure */
    public static function evict(string $k): void {
        unset(self::$store[$k]);
    }
}
===expect===
ImpureStaticPropertyAssignment@7:14-7:30: Assigning to static property Cache::$store in a @pure function
ImpureStaticPropertyAccess@7:20-7:26: Reading static property Cache::$store in a @pure function
