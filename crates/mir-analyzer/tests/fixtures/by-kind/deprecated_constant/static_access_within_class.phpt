===description===
DeprecatedConstant fires when static:: is used to access a deprecated constant from within the declaring class.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Config {
    /** @deprecated use MAX_RETRIES instead */
    const OLD_MAX = 3;

    public static function legacy(): void {
        echo static::OLD_MAX;
//                   ^^^^^^^ DeprecatedConstant: Constant Config::OLD_MAX is deprecated: use MAX_RETRIES instead
    }
}
===expect===
