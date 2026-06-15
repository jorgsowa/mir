===description===
DuplicateEnum fires when the same enum is declared twice.
===file===
<?php
enum Status {
    case Active;
}

enum Status {
    case Inactive;
}
===expect===
DuplicateEnum@6:0-8:1: Enum Status has already been defined
