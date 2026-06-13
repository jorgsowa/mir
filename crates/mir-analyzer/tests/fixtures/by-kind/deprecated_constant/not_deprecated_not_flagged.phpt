===description===
DeprecatedConstant does NOT fire for non-deprecated constants.
===file===
<?php
class Config {
    const MAX_RETRIES = 3;
}

$v = Config::MAX_RETRIES;
===expect===
