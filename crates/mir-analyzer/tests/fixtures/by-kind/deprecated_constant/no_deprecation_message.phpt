===description===
DeprecatedConstant fires with no message suffix when @deprecated tag has no text.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Config {
    /** @deprecated */
    const OLD_MAX = 3;
}

$v = Config::OLD_MAX;
===expect===
DeprecatedConstant@7:13-7:20: Constant Config::OLD_MAX is deprecated
