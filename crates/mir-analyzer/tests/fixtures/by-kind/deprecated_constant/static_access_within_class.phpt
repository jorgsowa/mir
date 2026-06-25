===description===
DeprecatedConstant fires when static:: is used to access a deprecated constant from within the declaring class.
===config===
suppress=UnusedVariable
===file===
<?php
class Config {
    /** @deprecated use MAX_RETRIES instead */
    const OLD_MAX = 3;

    public static function legacy(): void {
        echo static::OLD_MAX;
    }
}
===expect===
DeprecatedConstant@7:21-7:28: Constant Config::OLD_MAX is deprecated: use MAX_RETRIES instead
