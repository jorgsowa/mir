===description===
DeprecatedConstant fires when self:: is used to access a deprecated constant from within the declaring class.
===config===
suppress=UnusedVariable
===file===
<?php
class Config {
    /** @deprecated use MAX_RETRIES instead */
    const OLD_MAX = 3;

    public function legacy(): void {
        echo self::OLD_MAX;
    }
}
===expect===
DeprecatedConstant@7:19-7:26: Constant Config::OLD_MAX is deprecated: use MAX_RETRIES instead
