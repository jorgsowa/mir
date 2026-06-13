===description===
Regression (laravel/framework): a private static property read via `self::$prop`
counts as a use. The static-property-access path now resolves self/static/parent
through the FlowState and records the reference, so mir no longer reports
UnusedProperty (MimeType::$mime).
===config===
suppress=MissingClosureReturnType,MissingPropertyType,UnusedParam,UnusedVariable,MixedReturnStatement,MixedAssignment
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
===expect===

