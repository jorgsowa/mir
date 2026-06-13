===description===
DeprecatedConstant fires when accessing a deprecated class constant.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Config {
    /** @deprecated use MAX_RETRIES instead */
    const OLD_MAX = 3;
}

$v = Config::OLD_MAX;
===expect===
DeprecatedConstant@7:14-7:21: Constant Config::OLD_MAX is deprecated: use MAX_RETRIES instead
