===description===
A method declared `@return class-string<self>` returning `self::class` must
not be flagged InvalidReturnType — `self` inside `class-string<...>` was
never substituted with the declaring class, so the comparison always saw the
literal, unresolved "self" token instead of the actual class.
===file===
<?php

class Box {
    /** @return class-string<self> */
    public static function factory(): string {
        return self::class;
    }
}
===expect===
