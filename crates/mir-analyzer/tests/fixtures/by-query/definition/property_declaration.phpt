===description===
A property declaration name resolves to itself.
===cursor===
definition
===file===
<?php
class User {
    public string $na<CURSOR>me = "";
}
echo (new User)->name;
===expect===
test.php@3:4-3:28
