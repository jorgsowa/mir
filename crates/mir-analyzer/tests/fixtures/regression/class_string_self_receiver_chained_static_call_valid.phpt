===description===
The valid-call counterpart to class_string_self_receiver_chained_static_call
— a method that genuinely exists on the resolved class must still analyze
clean once `self` is substituted inside `class-string<self>`.
===file===
<?php

class Box {
    /** @return class-string<self> */
    public static function factory(): string {
        return self::class;
    }

    public static function build(): void {}
}

Box::factory()::build();
===expect===
