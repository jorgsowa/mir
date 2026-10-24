===description===
A method docblock written after its attributes applies, as in PHP.
===file===
<?php
final class Repo {
    #[\Deprecated]
    /** @return int */
    public function count() { return "x"; }
//                            ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
}
