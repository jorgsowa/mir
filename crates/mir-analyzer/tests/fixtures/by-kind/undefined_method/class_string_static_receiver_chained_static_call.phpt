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
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Box::doesNotExist() does not exist
===expect===
