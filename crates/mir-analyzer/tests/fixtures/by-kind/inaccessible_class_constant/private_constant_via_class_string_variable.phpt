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
//             ^^^^^^ InaccessibleClassConstant: Cannot access constant Config::SECRET
}
