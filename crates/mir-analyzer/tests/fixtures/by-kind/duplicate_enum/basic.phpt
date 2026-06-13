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
DuplicateEnum@6:1-8:2: Enum Status has already been defined
