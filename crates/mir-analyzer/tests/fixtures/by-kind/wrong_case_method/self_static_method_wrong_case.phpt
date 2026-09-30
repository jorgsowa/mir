===description===
Wrong case method name in self:: and static:: calls is reported.
===file===
<?php
class Factory {
    public static function create(): static { return new static(); }
    public function build(): void {
        self::CREATE();
//            ^^^^^^ WrongCaseMethod: Method name 'Factory::CREATE' has incorrect casing; use 'create'
        static::CREATE();
//              ^^^^^^ WrongCaseMethod: Method name 'Factory::CREATE' has incorrect casing; use 'create'
    }
}
===expect===
