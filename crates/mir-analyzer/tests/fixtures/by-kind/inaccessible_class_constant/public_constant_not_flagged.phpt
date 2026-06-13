===description===
InaccessibleClassConstant does NOT fire for public class constants.
===file===
<?php
class Config {
    public const TIMEOUT = 30;
}

$v = Config::TIMEOUT;
===expect===
