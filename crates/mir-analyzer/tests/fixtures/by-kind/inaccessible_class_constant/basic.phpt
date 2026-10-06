===description===
InaccessibleClassConstant fires when accessing a private class constant from outside.
===file===
<?php
class Config {
    private const SECRET = "hidden";
}

echo Config::SECRET;
//           ^^^^^^ InaccessibleClassConstant: Cannot access constant Config::SECRET
