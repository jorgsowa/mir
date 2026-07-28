===description===
Same false-negative as class_string_self_receiver_chained_static_call.phpt
but for `class-string<parent>`/`parent::class`. The reported class name
("Box") rather than the real parent ("Base") mirrors the same pre-existing
approximation bare `@return parent` already has — out of scope here, only
the previously-silent miss is what this fixture locks in.
===file===
<?php

class Base {
    public static function build(): void {}
}

class Box extends Base {
    /** @return class-string<parent> */
    public static function factory(): string {
        return parent::class;
    }
}

Box::factory()::doesNotExist();
===expect===
UndefinedMethod@14:0-14:30: Method Box::doesNotExist() does not exist
