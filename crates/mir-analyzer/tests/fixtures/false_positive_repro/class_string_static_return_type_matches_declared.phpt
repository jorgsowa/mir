===description===
Same as class_string_self_return_type_matches_declared.phpt but for
`class-string<static>` returning `static::class` — the same unresolved-token
bug affected `static`, not just `self`.
===file===
<?php

class Box {
    /** @return class-string<static> */
    public static function factory(): string {
        return static::class;
    }
}
===expect===
