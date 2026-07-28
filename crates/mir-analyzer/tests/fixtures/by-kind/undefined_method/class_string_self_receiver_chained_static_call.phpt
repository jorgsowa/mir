===description===
`class-string<self>` returned from one static call and immediately used as
the receiver of a chained `::` call must resolve against the declaring
class, not silently skip the check — `self` was never substituted inside
`class-string<...>`, so the receiver's class stayed the literal token "self"
and mir's self/static/parent leniency guard (for a truly unresolvable
context) swallowed the call instead of catching the real bug.
===file===
<?php

class Box {
    /** @return class-string<self> */
    public static function factory(): string {
        return self::class;
    }
}

Box::factory()::doesNotExist();
===expect===
UndefinedMethod@10:0-10:30: Method Box::doesNotExist() does not exist
