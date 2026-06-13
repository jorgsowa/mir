===description===
Laravel FP (laravel/framework): a private static property read via `self::$prop`
is not counted as a use, so mir's dead-code check reports UnusedProperty
(MimeType::$mime). Ignored pending fix — see ROADMAP §1.4 (liveness read-miss for
static-property access).
===ignore===
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,MixedReturnStatement,MixedAssignment
===file===
<?php
class MimeTypes {}
class MimeType {
    /** @var MimeTypes|null */
    private static $mime;

    public static function getMimeTypes(): MimeTypes {
        if (self::$mime === null) {
            self::$mime = new MimeTypes();
        }
        return self::$mime;
    }
}
