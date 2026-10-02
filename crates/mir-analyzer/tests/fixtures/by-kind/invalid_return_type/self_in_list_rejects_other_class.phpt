===description===
list<self> still rejects elements of an unrelated class
===file===
<?php
class Node {
    /** @return list<self> */
    public static function make(): array {
        return [new Other()];
//      ^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array{0: Other}' is not compatible with declared 'list<self(Node)>'
    }
}
class Other {}
===expect===
