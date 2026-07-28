===description===
Same false-negative as class_string_self_receiver_chained_static_call.phpt
but for `class-string<static>`/`static::class`.
===file===
<?php

class Box {
    /** @return class-string<static> */
    public static function factory(): string {
        return static::class;
    }
}

Box::factory()::doesNotExist();
===expect===
UndefinedMethod@10:0-10:30: Method Box::doesNotExist() does not exist
