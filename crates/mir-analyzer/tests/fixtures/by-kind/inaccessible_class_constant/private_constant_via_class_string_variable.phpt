===description===
$cls::SECRET (constant access through a class-string variable) reports InaccessibleClassConstant for a private constant accessed from outside its class.
===file===
<?php
class Config {
    private const SECRET = "hidden";
}
function run(): void {
    $cls = Config::class;
    echo $cls::SECRET;
}
===expect===
InaccessibleClassConstant@7:15-7:21: Cannot access constant Config::SECRET
