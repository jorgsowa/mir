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
//                 ^^^^^^^ DeprecatedConstant: Constant Config::OLD_MAX is deprecated: use MAX_RETRIES instead
    }
}
===expect===
