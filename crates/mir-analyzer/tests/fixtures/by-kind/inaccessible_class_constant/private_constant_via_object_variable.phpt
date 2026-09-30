===description===
$obj::SECRET (constant access through an object-instance variable) reports InaccessibleClassConstant for a private constant accessed from outside its class.
===file===
<?php
class Config {
    private const SECRET = "hidden";
}
function run(Config $c): void {
    echo $c::SECRET;
//           ^^^^^^ InaccessibleClassConstant: Cannot access constant Config::SECRET
}
===expect===
