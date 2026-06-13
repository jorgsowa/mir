===description===
DeprecatedConstant does NOT fire for non-deprecated constants.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Config {
    const MAX_RETRIES = 3;
}

$v = Config::MAX_RETRIES;
===expect===
