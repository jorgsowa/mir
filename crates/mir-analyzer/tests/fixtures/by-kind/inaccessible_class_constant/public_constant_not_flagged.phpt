===description===
InaccessibleClassConstant does NOT fire for public class constants.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Config {
    public const TIMEOUT = 30;
}

$v = Config::TIMEOUT;
===expect===
