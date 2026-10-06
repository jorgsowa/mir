===description===
NonStaticSelfCall fires regardless of visibility — a protected non-static method called via self:: in a static context is still reported.
===file===
<?php
class Validator {
    protected function validate(): bool { return true; }

    public static function check(): bool {
        return self::validate();
//             ^^^^^^^^^^^^^^^^ NonStaticSelfCall: Non-static method Validator::validate() cannot be called statically
    }
}
