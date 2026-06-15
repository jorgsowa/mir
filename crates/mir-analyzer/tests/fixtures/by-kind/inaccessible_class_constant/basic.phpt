===description===
InaccessibleClassConstant fires when accessing a private class constant from outside.
===file===
<?php
class Config {
    private const SECRET = "hidden";
}

echo Config::SECRET;
===expect===
InaccessibleClassConstant@6:13-6:19: Cannot access constant Config::SECRET
