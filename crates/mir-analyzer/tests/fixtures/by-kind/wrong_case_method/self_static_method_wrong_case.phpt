===description===
Wrong case method name in self:: and static:: calls is reported.
===file===
<?php
class Factory {
    public static function create(): static { return new static(); }
    public function build(): void {
        self::CREATE();
        static::CREATE();
    }
}
===expect===
WrongCaseMethod@5:15-5:21: Method name 'Factory::CREATE' has incorrect casing; use 'create'
WrongCaseMethod@6:17-6:23: Method name 'Factory::CREATE' has incorrect casing; use 'create'
