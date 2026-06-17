===description===
wrong argument type via bare FQN static call is still caught
===file:Validator.php===
<?php
class Validator {
    public static function check(string $value): bool { return strlen($value) > 0; }
}
===file:App.php===
<?php
function run(): void {
    \Validator::check(42);
}
===expect===
App.php: ArgumentTypeCoercion@3:22-3:24: Argument $value of check() expects 'string', got '42' — coercion may fail at runtime
