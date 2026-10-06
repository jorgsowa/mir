===description===
DuplicateEnum fires when the same enum is declared twice.
===file===
<?php
enum Status {
    case Active;
}

enum Status {
//<^ +2:1 DuplicateEnum: Enum Status has already been defined
    case Inactive;
}
